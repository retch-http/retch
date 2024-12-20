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
pub use request::RequestOptions;