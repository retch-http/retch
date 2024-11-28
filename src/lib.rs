mod http_headers;
mod tls;
pub mod retcher;

#[derive(PartialEq, Debug, Clone, Copy, Default)]
pub enum Browser {
  #[default]
  Chrome,
  Firefox,
}

