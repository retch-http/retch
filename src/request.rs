use std::{collections::HashMap, time::Duration};

/// RequestOptions is a struct holding additional options for the fetch request.
#[derive(Debug, Clone)]
pub struct RequestOptions {
  /// A `HashMap` that holds custom HTTP headers. These are added to the default headers and should never overwrite them.
  pub headers: HashMap<String, String>,
  pub timeout: Option<Duration>,
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