mod statics;
mod ffdhe;

use crate::Browser;
use rustls::client::{BrowserEmulator as RusTLSBrowser, BrowserType, EchGreaseConfig};
use rustls::crypto::aws_lc_rs::kx_group::{SECP256R1, SECP384R1, X25519};
use rustls::crypto::CryptoProvider;
use rustls::RootCertStore;

pub struct TlsConfig {}

impl TlsConfig {
  pub fn builder() -> TlsConfigBuilder {
      TlsConfigBuilder::default()
  }
}

#[derive(Debug, Clone, Copy)]
pub struct TlsConfigBuilder {
  browser: Option<Browser>,
}

impl Default for TlsConfigBuilder {
  fn default() -> Self {
      TlsConfigBuilder {
          browser: Some(Browser::Chrome),
      }
  }
}

impl TlsConfigBuilder {
  fn get_ech_mode(self) -> rustls::client::EchMode {
      let (public_key, _) = statics::GREASE_HPKE_SUITE
          .generate_key_pair()
          .unwrap();
      
      EchGreaseConfig::new(statics::GREASE_HPKE_SUITE, public_key).into()
  }

  pub fn with_browser(&mut self, browser: Option<Browser>) -> &mut Self {
      self.browser = browser;
      self
  }

  pub fn build(self) -> rustls::ClientConfig {
    let mut root_store = RootCertStore::empty();
    root_store.extend(
        webpki_roots::TLS_SERVER_ROOTS.iter().cloned(),
    );

    match self.browser {
      Some(browser) => {
        let rustls_browser = match browser {
          Browser::Chrome => RusTLSBrowser { browser_type: BrowserType::Chrome, version: 125 },
          Browser::Firefox => RusTLSBrowser { browser_type: BrowserType::Firefox, version: 125 },
        };

        let mut crypto_provider = CryptoProvider::builder()
            .with_browser_emulator(&rustls_browser)
            .build();

        match browser {
          Browser::Firefox => {
            crypto_provider.kx_groups = vec![
              X25519,
              SECP256R1,
              SECP384R1,
              // TODO : add SECPR521R1
              &ffdhe::FFDHE2048_KX_GROUP, 
              &ffdhe::FFDHE3072_KX_GROUP,
            ];
          },
          _ => {}
        }

        let config: rustls::ClientConfig = rustls::ClientConfig::builder_with_provider(
                crypto_provider.into(),
            )
            // TODO - use the ECH extension consistently
            .with_ech(self.get_ech_mode()).unwrap()
            .with_root_certificates(root_store)
            .with_browser_emulator(&rustls_browser)
            .with_no_client_auth();
    
        config
      },
      None => {
        let crypto_provider = CryptoProvider::builder()
            .build();

        let config: rustls::ClientConfig = rustls::ClientConfig::builder_with_provider(
                crypto_provider.into(),
            )
            // TODO - use the ECH extension consistently
            .with_ech(self.get_ech_mode()).unwrap()
            .with_root_certificates(root_store)
            .with_no_client_auth();
    
        config
      }
    }
  }
}