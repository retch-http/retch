use std::{collections::HashMap, time::Duration};

#[derive(Debug, Clone)]
pub struct RequestOptions {
  /// A `HashMap` that holds custom HTTP headers. These are added to the default headers and should never overwrite them.
  pub headers: HashMap<String, String>,
  /// The timeout for the request. This option overrides the global `Retcher` timeout.
  pub timeout: Option<Duration>,
  /// Whether to use HTTP/3 with prior knowledge. This can cause issues with servers that don't support HTTP/3.
  pub http3_prior_knowledge: bool,
}

impl Default for RequestOptions {
  fn default() -> Self {
    RequestOptions {
      headers: HashMap::new(),
      timeout: None,
      http3_prior_knowledge: false,
    }
  }
}