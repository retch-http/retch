mod statics;

use crate::Browser;
use rustls::client::{BrowserEmulator as RusTLSBrowser, EchGreaseConfig};
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
  browser: Browser,
}

impl Default for TlsConfigBuilder {
  fn default() -> Self {
      TlsConfigBuilder {
          browser: Browser::Chrome,
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

  pub fn with_browser(&mut self, browser: Browser) -> &mut Self {
      self.browser = browser;
      self
  }

  pub fn build(self) -> rustls::ClientConfig {
      let crypto_provider = CryptoProvider::builder()
        .with_browser_emulator(match self.browser {
            Browser::Chrome => RusTLSBrowser::Chrome,
        })
        .build();

      let mut root_store = RootCertStore::empty();
      root_store.extend(
          webpki_roots::TLS_SERVER_ROOTS.iter().cloned(),
      );

      let config: rustls::ClientConfig = rustls::ClientConfig::builder_with_provider(
              crypto_provider.into(),
          )
          // TODO - use the ECH extension consistently
          .with_ech(self.get_ech_mode()).unwrap()
          .with_root_certificates(root_store)
          .with_browser_emulator(match self.browser {
            Browser::Chrome => RusTLSBrowser::Chrome,
            })
          .with_no_client_auth();
  
      config
  }
}