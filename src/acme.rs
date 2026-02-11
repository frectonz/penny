use color_eyre::eyre::{Context, eyre};
use instant_acme::{
    Account, AccountCredentials, AuthorizationStatus, ChallengeType, Identifier, LetsEncrypt,
    NewAccount, NewOrder, OrderStatus,
};
use rcgen::{CertificateParams, DistinguishedName, KeyPair};
use tracing::{debug, info};

use crate::challenge::{ChallengeStore, add_challenge, remove_challenge};
use crate::config::TlsConfig;
use crate::db::SqliteDatabase;

async fn cleanup_pending_challenges(challenge_store: &ChallengeStore, tokens: &[String]) {
    for token in tokens {
        remove_challenge(challenge_store, token).await;
    }
}

/// ACME client for obtaining and managing certificates.
pub struct AcmeClient {
    account: Account,
    staging: bool,
    order_poll_interval_secs: u64,
    order_poll_max_retries: u32,
    cert_poll_interval_secs: u64,
    cert_poll_max_retries: u32,
}

impl AcmeClient {
    /// Creates a new ACME client, loading or creating an account as needed.
    pub async fn new(config: &TlsConfig, db: &SqliteDatabase) -> color_eyre::Result<Self> {
        let account = match db.get_acme_account().await? {
            Some(pem) => {
                info!("loading existing ACME account");
                Self::load_account(&pem).await?
            }
            None => {
                info!("creating new ACME account");
                let (account, pem) =
                    Self::create_account(&config.acme_email, config.staging).await?;
                db.save_acme_account(&pem).await?;
                account
            }
        };

        Ok(Self {
            account,
            staging: config.staging,
            order_poll_interval_secs: config.order_poll_interval_secs,
            order_poll_max_retries: config.order_poll_max_retries,
            cert_poll_interval_secs: config.cert_poll_interval_secs,
            cert_poll_max_retries: config.cert_poll_max_retries,
        })
    }

    /// Creates a new ACME account and returns it along with the private key PEM.
    async fn create_account(email: &str, staging: bool) -> color_eyre::Result<(Account, String)> {
        let url = if staging {
            LetsEncrypt::Staging.url()
        } else {
            LetsEncrypt::Production.url()
        };

        let (account, credentials) = Account::builder()
            .wrap_err("failed to create ACME account builder")?
            .create(
                &NewAccount {
                    contact: &[&format!("mailto:{}", email)],
                    terms_of_service_agreed: true,
                    only_return_existing: false,
                },
                url.to_string(),
                None,
            )
            .await
            .wrap_err("failed to create ACME account")?;

        let pem =
            serde_json::to_string(&credentials).wrap_err("failed to serialize ACME credentials")?;

        Ok((account, pem))
    }

    /// Loads an existing ACME account from credentials PEM.
    async fn load_account(pem: &str) -> color_eyre::Result<Account> {
        let credentials: AccountCredentials =
            serde_json::from_str(pem).wrap_err("failed to deserialize ACME credentials")?;

        Account::builder()
            .wrap_err("failed to create ACME account builder")?
            .from_credentials(credentials)
            .await
            .wrap_err("failed to load ACME account")
    }

    /// Requests a certificate for the given domains.
    /// Returns the certificate and private key as PEM strings.
    pub async fn obtain_certificate(
        &self,
        domains: &[&str],
        challenge_store: &ChallengeStore,
    ) -> color_eyre::Result<(String, String)> {
        if domains.is_empty() {
            return Err(eyre!("no domains provided"));
        }

        info!(domains = ?domains, staging = self.staging, "requesting certificate");

        // Create identifiers for all domains
        let identifiers: Vec<Identifier> = domains
            .iter()
            .map(|d| Identifier::Dns((*d).to_owned()))
            .collect();

        // Create the order
        let mut order = self
            .account
            .new_order(&NewOrder::new(&identifiers))
            .await
            .wrap_err("failed to create ACME order")?;

        let state = order.state();
        debug!(status = ?state.status, "order created");

        // Get authorizations and set up challenges
        let mut pending_tokens = Vec::new();

        let mut auths = order.authorizations();
        while let Some(auth_result) = auths.next().await {
            let mut auth = auth_result.wrap_err("failed to get authorization")?;

            match auth.status {
                AuthorizationStatus::Valid => {
                    debug!(identifier = ?auth.identifier(), "authorization already valid");
                    continue;
                }
                AuthorizationStatus::Pending => {
                    debug!(identifier = ?auth.identifier(), "authorization pending, setting up challenge");
                }
                status => {
                    return Err(eyre!("unexpected authorization status: {:?}", status));
                }
            }

            let mut challenge = auth
                .challenge(ChallengeType::Http01)
                .ok_or_else(|| eyre!("no HTTP-01 challenge found"))?;

            let token = challenge.token.clone();
            let key_auth = challenge.key_authorization().as_str().to_owned();

            add_challenge(challenge_store, token.clone(), key_auth).await;
            pending_tokens.push(token);

            challenge
                .set_ready()
                .await
                .wrap_err("failed to set challenge ready")?;
        }
        // Wait for order to become ready
        let mut tries = 0;
        let max_tries = self.order_poll_max_retries;
        let _state = loop {
            tokio::time::sleep(std::time::Duration::from_secs(
                self.order_poll_interval_secs,
            ))
            .await;
            let state = order.refresh().await.wrap_err("failed to refresh order")?;

            debug!(status = ?state.status, tries = tries, "checking order status");

            match state.status {
                OrderStatus::Ready => break state,
                OrderStatus::Invalid => {
                    cleanup_pending_challenges(challenge_store, &pending_tokens).await;
                    return Err(eyre!("order became invalid"));
                }
                OrderStatus::Valid => break state,
                OrderStatus::Pending => {
                    tries += 1;
                    if tries >= max_tries {
                        cleanup_pending_challenges(challenge_store, &pending_tokens).await;
                        return Err(eyre!("order did not become ready in time"));
                    }
                }
                OrderStatus::Processing => {
                    tries += 1;
                    if tries >= max_tries {
                        cleanup_pending_challenges(challenge_store, &pending_tokens).await;
                        return Err(eyre!("order processing timed out"));
                    }
                }
            }
        };

        cleanup_pending_challenges(challenge_store, &pending_tokens).await;

        // Generate CSR
        let key_pair = KeyPair::generate().wrap_err("failed to generate key pair")?;
        let private_key_pem = key_pair.serialize_pem();

        let domain_strings: Vec<String> = domains.iter().map(|s| (*s).to_owned()).collect();
        let mut params = CertificateParams::new(domain_strings)
            .wrap_err("failed to create certificate params")?;
        params.distinguished_name = DistinguishedName::new();

        let csr = params
            .serialize_request(&key_pair)
            .wrap_err("failed to serialize CSR")?;

        // Finalize order with CSR
        order
            .finalize_csr(csr.der())
            .await
            .wrap_err("failed to finalize order")?;

        // Wait for certificate
        let mut tries = 0;
        let cert_chain_pem = loop {
            tokio::time::sleep(std::time::Duration::from_secs(self.cert_poll_interval_secs)).await;

            match order
                .certificate()
                .await
                .wrap_err("failed to get certificate")?
            {
                Some(cert) => break cert,
                None => {
                    tries += 1;
                    if tries >= self.cert_poll_max_retries {
                        return Err(eyre!("certificate not ready in time"));
                    }
                }
            }
        };

        info!(domains = ?domains, "certificate obtained successfully");

        Ok((cert_chain_pem, private_key_pem))
    }
}
