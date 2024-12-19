mod http_headers;
mod tls;
pub mod retcher;

#[derive(PartialEq, Debug, Clone, Copy, Default)]
pub enum Browser {
  #[default]
  Chrome,
  Firefox,
}

pub(crate) mod errors;
pub(crate) mod utils;
pub(crate) mod http3;
pub(crate) mod logger;
pub mod request;
