use std::{collections::HashMap, time::Duration};

/// A struct that holds the request options.
/// 
/// Unlike the [`RetcherBuilder`](crate::retcher::RetcherBuilder) struct, these options are specific to a single request.
/// 
/// Used by the [`Retcher`](crate::retcher::Retcher) struct's methods.
#[derive(Debug, Clone)]
pub struct RequestOptions {
  /// A `HashMap` that holds custom HTTP headers. These are added to the default headers and should never overwrite them.
  pub headers: HashMap<String, String>,
  /// The timeout for the request. This option overrides the global `Retcher` timeout.
  pub timeout: Option<Duration>,
  /// Enforce the use of HTTP/3 for this request. This will cause broken responses from servers that don't support HTTP/3.
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