// crates/services/web-server/src/tls.rs

use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use rustls_pemfile::{certs, pkcs8_private_keys};
use tokio::sync::watch;
use tokio_rustls::rustls::{self, Certificate, PrivateKey};
use tracing::{error, info};

use crate::error::{Error, Result};

pub struct TlsConfig {
    cert_path: PathBuf,
    key_path: PathBuf,
}

impl TlsConfig {
    pub fn new(domain: &str) -> Self {
        // Standard Let's Encrypt paths
        Self {
            cert_path: PathBuf::from(format!("/etc/letsencrypt/live/{}/fullchain.pem", domain)),
            key_path: PathBuf::from(format!("/etc/letsencrypt/live/{}/privkey.pem", domain)),
        }
    }

    pub async fn load_config(&self) -> Result<rustls::ServerConfig> {
        // Load certificate and private key files
        let cert_file = File::open(&self.cert_path)
            .map_err(|e| Error::Tls(format!("Failed to open cert file: {}", e)))?;
        let key_file = File::open(&self.key_path)
            .map_err(|e| Error::Tls(format!("Failed to open key file: {}", e)))?;
        
        let cert_reader = &mut BufReader::new(cert_file);
        let key_reader = &mut BufReader::new(key_file);

        // Convert certificates and private key
        let certs: Vec<Certificate> = certs(cert_reader)?
            .into_iter()
            .map(Certificate)
            .collect();
        
        let keys: Vec<PrivateKey> = pkcs8_private_keys(key_reader)?
            .into_iter()
            .map(PrivateKey)
            .collect();

        // Create TLS configuration
        let config = rustls::ServerConfig::builder()
            .with_safe_defaults()
            .with_no_client_auth()
            .with_single_cert(certs, keys[0].clone())
            .map_err(|err| Error::Tls(format!("TLS config error: {}", err)))?;

        Ok(config)
    }

    pub async fn watch_cert_changes(
        self,
        config_tx: watch::Sender<Arc<rustls::ServerConfig>>,
    ) -> Result<()> {
        let (tx, rx) = std::sync::mpsc::channel();
        let mut watcher: RecommendedWatcher = Watcher::new(tx, notify::Config::default())?;

        // Watch certificate directory
        watcher.watch(
            self.cert_path.parent().unwrap(),
            RecursiveMode::NonRecursive,
        )?;

        for res in rx {
            match res {
                Ok(_) => {
                    // Reload TLS config when certificates change
                    match self.load_config().await {
                        Ok(new_config) => {
                            let _ = config_tx.send(Arc::new(new_config));
                            info!("TLS configuration reloaded successfully");
                        }
                        Err(e) => error!("Failed to reload TLS config: {}", e),
                    }
                }
                Err(e) => error!("Watch error: {}", e),
            }
        }
        Ok(())
    }
}