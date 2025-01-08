//! # retch | browser impersonation made simple
//! 
//! Retch is a `rust` library that allows you to impersonate a browser and make requests to websites. It is built on top of `reqwest`, `rustls` and `tokio` and supports HTTP/1.1, HTTP/2, and HTTP/3.
//! 
//! 
//! ### ⚠️ Warning ⚠️
//!
//! Technically speaking, the `retch` project is a somewhat thin wrapper around `reqwest` that provides a more ergonomic API for making requests to websites. 
//! The real strength of `retch` is that it uses patched versions of `rustls` and other libraries that allow it to make browser-like requests.
//! 
//! Note that if you want to use this library in your rust project, you have to add the following dependencies to your `Cargo.toml` file:
//! ```toml
//! 
//! [dependencies]
//! retch = { path="../retch" }
//! rustls = { version="0.23.16" }
//! tokio = { version="1.41.1", features = ["full"] }
//! 
//! [patch.crates-io]
//! rustls = { path="../rustls/rustls/" }
//! reqwest = { path="../reqwest/" }
//! h2 = { path="../h2/" }
//! ```
//! 
//! 
//! 

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