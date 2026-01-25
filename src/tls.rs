use std::fs;
use std::path::{Path, PathBuf};

use color_eyre::eyre::{Context, eyre};
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
                let renewal_threshold = jiff::Span::new().days(renewal_days as i64);

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

/// Sanitizes a domain name for use as a filename.
fn sanitize_domain(domain: &str) -> String {
    domain.replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "_")
}
