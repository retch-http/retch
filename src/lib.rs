//! # retch | browser impersonation made simple
//! 
//! Retch is a `rust` library that allows you to impersonate a browser and make requests to websites. It is built on top of `reqwest`, `rustls` and `tokio` and supports HTTP/1.1, HTTP/2, and HTTP/3.
//! 
//! The library provides a simple API for making requests to websites, and it also allows you to customize the request headers, use proxies, custom timeouts and more.
//! 
//! ```rust
//! use retch::retcher::Retcher;
//! 
//! #[tokio::main]
//! async fn main() {
//!    let retcher = Retcher::new().unwrap();
//!    let response = retcher.get("https://example.com").send().await.unwrap();
//! 
//!    println!("{}", response.text().await.unwrap());
//! }
//! ```
//! 
//! ### Other projects
//! 
//! If you are looking for a command-line tool that allows you to make requests to websites, check out the [`retch-cli`](https://github.com/retch-http/retch-cli/) project.
//! 
//! If you'd prefer to use `retch` from a Node.js application, check out the [`retch-node`](https://github.com/retch-http/retch-node) repository, or download the package from npm:
//! ```bash
//! npm install retch-http
//! ```
//! 
//! ### Usage from Rust
//!
//! Technically speaking, the `retch` project is a somewhat thin wrapper around `reqwest` that provides a more ergonomic API for making requests to websites. 
//! The real strength of `retch` is that it uses patched versions of `rustls` and other libraries that allow it to make browser-like requests.
//! 
//! Note that if you want to use this library in your rust project, you have to add the following dependencies to your `Cargo.toml` file:
//! ```toml
//! [dependencies]
//! retch = { git="https://github.com/retch-http/retch.git", branch="master" }
//! 
//! [patch.crates-io]
//! rustls = { git="https://github.com/retch-http/rustls.git", branch="retch-patch" }
//! h2 = { git="https://github.com/retch-http/h2.git", branch="retch-patch" }
//! ```
//! 
//! Without the patched dependencies, the project won't build.
//! 
//! Note that you also have to build your project with `rustflags = "--cfg reqwest_unstable"`, otherwise, the build will also fail.
//! This is because `retch` uses unstable features of `reqwest` (namely `http3` support), which are not available in the stable version of the library.

#![deny(unused_crate_dependencies)]
mod http_headers;
mod tls;
mod response_parsing;

pub(crate) mod http3;

/// Main module that contains the `Retcher` struct and its methods.
pub mod retcher;

/// Customizing request options.
pub mod request;

/// Contains browser emulation-related types and functions.
pub mod emulation {
  
  /// The `Browser` enum is used to specify the browser that should be impersonated.
  /// 
  /// It can be passed as a parameter to [`RetcherBuilder::with_browser`](crate::retcher::RetcherBuilder::with_browser) 
  /// to use the browser emulation with the built [`Retcher`](crate::retcher::Retcher) instance.
  #[derive(PartialEq, Debug, Clone, Copy, Default)]
  pub enum Browser {
    #[default]
    Chrome,
    Firefox,
  }
}

/// Various utility functions and types.
pub mod utils {
  pub use crate::response_parsing::decode;
  pub use crate::response_parsing::ContentType;
  pub use encoding::all as encodings;
}