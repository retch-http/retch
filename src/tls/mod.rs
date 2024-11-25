mod statics;

use crate::Browser;
use rustls::client::EchGreaseConfig;
use rustls::crypto::{CryptoProvider, aws_lc_rs};
use rustls::RootCertStore;

use std::sync::Arc;

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
      let (cipher_suites, signature_algorithms) = match self.browser {
          Browser::Chrome => (statics::CHROME_CIPHER_SUITES, statics::CHROME_SIGNATURE_VERIFICATION_ALGOS),
      };

      let crypto_provider = CryptoProvider {
          cipher_suites: cipher_suites.into(),
          signature_verification_algorithms: signature_algorithms,
          ..aws_lc_rs::default_provider()
      };

      let mut root_store = RootCertStore::empty();
      root_store.extend(
          webpki_roots::TLS_SERVER_ROOTS.iter().cloned(),
      );

      let mut config: rustls::ClientConfig= rustls::ClientConfig::builder_with_provider(
              crypto_provider.into(),
          )
          // TODO - use the ECH extension consistently
          .with_safe_default_protocol_versions().unwrap()
          .with_root_certificates(root_store)
          .with_no_client_auth();
      
      config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];
      config.key_log = Arc::new(rustls::KeyLogFile::new());
      config.cert_decompressors = vec![rustls::compress::BROTLI_DECOMPRESSOR];
      config.cert_compressors = vec![rustls::compress::BROTLI_COMPRESSOR];
  
      config
  }
}