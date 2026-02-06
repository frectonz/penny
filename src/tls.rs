use std::fs;
use std::path::{Path, PathBuf};

use async_trait::async_trait;
use color_eyre::eyre::{Context, eyre};
use pingora::tls::ext;
use pingora::tls::pkey::PKey;
use pingora::tls::ssl::{NameType, SslRef};
use pingora::tls::x509::X509;
use tracing::{debug, info, warn};
use x509_parser::prelude::*;

/// Manages certificate storage on the filesystem.
pub struct CertificateStore {
    certs_dir: PathBuf,
}

impl CertificateStore {
    /// Creates a new certificate store with the given directory.
    /// Creates the directory if it doesn't exist.
    pub fn new(certs_dir: &Path) -> color_eyre::Result<Self> {
        if !certs_dir.exists() {
            fs::create_dir_all(certs_dir)
                .wrap_err_with(|| format!("failed to create certs directory: {:?}", certs_dir))?;
            info!(path = ?certs_dir, "created certificates directory");
        }

        Ok(Self {
            certs_dir: certs_dir.to_path_buf(),
        })
    }

    /// Gets the certificate and key file paths for a domain.
    /// Returns None if the certificate doesn't exist.
    pub fn get_certificate(&self, domain: &str) -> Option<(PathBuf, PathBuf)> {
        let cert_path = self.cert_path(domain);
        let key_path = self.key_path(domain);

        if cert_path.exists() && key_path.exists() {
            Some((cert_path, key_path))
        } else {
            None
        }
    }

    /// Stores a certificate and private key for a domain.
    pub fn store_certificate(
        &self,
        domain: &str,
        cert_pem: &str,
        key_pem: &str,
    ) -> color_eyre::Result<()> {
        let cert_path = self.cert_path(domain);
        let key_path = self.key_path(domain);

        fs::write(&cert_path, cert_pem)
            .wrap_err_with(|| format!("failed to write certificate: {:?}", cert_path))?;

        fs::write(&key_path, key_pem)
            .wrap_err_with(|| format!("failed to write private key: {:?}", key_path))?;

        info!(domain = %domain, cert_path = ?cert_path, "stored certificate");

        Ok(())
    }

    /// Checks if a certificate needs renewal.
    /// Returns true if the certificate expires within `renewal_days` days,
    /// or if the certificate doesn't exist.
    pub fn needs_renewal(&self, domain: &str, renewal_days: u32) -> bool {
        let cert_path = self.cert_path(domain);

        if !cert_path.exists() {
            debug!(domain = %domain, "certificate does not exist, needs provisioning");
            return true;
        }

        match self.get_expiry(&cert_path) {
            Ok(expiry) => {
                let now = jiff::Timestamp::now();
                let renewal_threshold = jiff::Span::new().hours(renewal_days as i64 * 24);

                match now.checked_add(renewal_threshold) {
                    Ok(threshold) => {
                        let needs_renewal = expiry < threshold;
                        if needs_renewal {
                            info!(
                                domain = %domain,
                                expiry = %expiry,
                                "certificate expires soon, needs renewal"
                            );
                        }
                        needs_renewal
                    }
                    Err(_) => {
                        warn!(domain = %domain, "failed to calculate renewal threshold");
                        true
                    }
                }
            }
            Err(e) => {
                warn!(domain = %domain, error = %e, "failed to get certificate expiry");
                true
            }
        }
    }

    /// Gets the expiry timestamp of a certificate.
    fn get_expiry(&self, cert_path: &Path) -> color_eyre::Result<jiff::Timestamp> {
        let pem_data = fs::read(cert_path)
            .wrap_err_with(|| format!("failed to read certificate: {:?}", cert_path))?;

        // Parse the first certificate in the chain
        let pems = ::pem::parse_many(&pem_data).wrap_err("failed to parse PEM")?;
        let first_pem = pems.first().ok_or_else(|| eyre!("no PEM found in file"))?;

        let (_, cert) = X509Certificate::from_der(first_pem.contents())
            .map_err(|e| eyre!("failed to parse X509 certificate: {:?}", e))?;

        let not_after = cert.validity().not_after;
        let timestamp = jiff::Timestamp::from_second(not_after.timestamp())
            .wrap_err("failed to convert timestamp")?;

        debug!(cert_path = ?cert_path, expiry = %timestamp, "parsed certificate expiry");

        Ok(timestamp)
    }

    /// Returns the path to the certificate file for a domain.
    fn cert_path(&self, domain: &str) -> PathBuf {
        self.certs_dir
            .join(format!("{}.crt", sanitize_domain(domain)))
    }

    /// Returns the path to the private key file for a domain.
    fn key_path(&self, domain: &str) -> PathBuf {
        self.certs_dir
            .join(format!("{}.key", sanitize_domain(domain)))
    }
}

/// Resolves certificates from disk on each TLS handshake via SNI.
/// This ensures newly provisioned or renewed certificates are picked up
/// without requiring a restart.
pub struct DynamicCertificates {
    cert_store: CertificateStore,
}

impl DynamicCertificates {
    pub fn new(cert_store: CertificateStore) -> Self {
        Self { cert_store }
    }
}

#[async_trait]
impl pingora::listeners::TlsAccept for DynamicCertificates {
    async fn certificate_callback(&self, ssl: &mut SslRef) {
        let domain = match ssl.servername(NameType::HOST_NAME) {
            Some(name) => name.to_owned(),
            None => {
                warn!("TLS handshake without SNI hostname");
                return;
            }
        };

        let (cert_path, key_path) = match self.cert_store.get_certificate(&domain) {
            Some(paths) => paths,
            None => {
                warn!(domain = %domain, "no certificate for requested domain");
                return;
            }
        };

        let cert_bytes = match fs::read(&cert_path) {
            Ok(bytes) => bytes,
            Err(e) => {
                warn!(domain = %domain, error = %e, "failed to read certificate");
                return;
            }
        };
        let key_bytes = match fs::read(&key_path) {
            Ok(bytes) => bytes,
            Err(e) => {
                warn!(domain = %domain, error = %e, "failed to read private key");
                return;
            }
        };

        let cert = match X509::from_pem(&cert_bytes) {
            Ok(cert) => cert,
            Err(e) => {
                warn!(domain = %domain, error = %e, "failed to parse certificate");
                return;
            }
        };
        let key = match PKey::private_key_from_pem(&key_bytes) {
            Ok(key) => key,
            Err(e) => {
                warn!(domain = %domain, error = %e, "failed to parse private key");
                return;
            }
        };

        if let Err(e) = ext::ssl_use_certificate(ssl, &cert) {
            warn!(domain = %domain, error = %e, "failed to set certificate");
            return;
        }
        if let Err(e) = ext::ssl_use_private_key(ssl, &key) {
            warn!(domain = %domain, error = %e, "failed to set private key");
        }
    }
}

/// Sanitizes a domain name for use as a filename.
fn sanitize_domain(domain: &str) -> String {
    domain.replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "_")
}
