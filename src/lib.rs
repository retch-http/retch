#![deny(unused_crate_dependencies)]
mod http_headers;
mod tls;
pub mod retcher;

#[derive(PartialEq, Debug, Clone, Copy, Default)]
pub enum Browser {
  #[default]
  Chrome,
  Firefox,
}

pub(crate) mod http3;
pub(crate) mod request;
mod response_parsing;

pub use request::RequestOptions;

pub mod utils {
  pub use crate::response_parsing::decode;
  pub use crate::response_parsing::ContentType;
  pub use encoding::all as encodings;
}