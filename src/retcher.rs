use std::collections::HashMap;
use reqwest::Response;
use url::Url;

use crate::{http_headers::HttpHeaders, tls};
use super::Browser;

#[derive(Debug, Clone)]
pub enum ErrorType {
  UrlParseError,
  ProtocolError,
  RequestError,
  ResponseError,
}

struct RetcherConfig {
  browser: Option<Browser>,
}

/// Retcher is the main struct used to make (impersonated) requests.
/// 
/// It uses `reqwest::Client` to make requests and holds info about the impersonated browser.
pub struct Retcher {
  client: reqwest::Client,
  config: RetcherConfig,
}

#[derive(Debug, Clone, Copy)]
pub struct RetcherBuilder {
  browser: Option<Browser>,
  ignore_tls_errors: bool,
}

impl Default for RetcherBuilder {
  fn default() -> Self {
    RetcherBuilder {
      browser: None,
      ignore_tls_errors: false,
    }
  }
}

impl RetcherBuilder {
  pub fn with_browser(&mut self, browser: Browser) -> &mut Self {
    self.browser = Some(browser);
    self
  }

  pub fn with_ignore_tls_errors(&mut self, ignore_tls_errors: bool) -> &mut Self {
    self.ignore_tls_errors = ignore_tls_errors;
    self
  }

  pub fn build(self) -> Retcher {
    Retcher::new(self)
  }
}

/// RequestOptions is a struct holding additional options for the fetch request.
pub struct RequestOptions {
  /// A `HashMap` that holds custom HTTP headers. These are added to the default headers and should never overwrite them.
  pub headers: HashMap<String, String>
}

impl Default for RequestOptions {
  fn default() -> Self {
    RequestOptions {
      headers: HashMap::new(),
    }
  }
}

impl Retcher {
  /// Creates a new `Retcher` instance with the given `EngineOptions`.
  fn new(options: RetcherBuilder) -> Self {
    let mut client = reqwest::Client::builder();
    let tls_config = tls::TlsConfig::builder()
      .with_browser(options.browser)
      .build();
    
    client = client
      .danger_accept_invalid_certs(options.ignore_tls_errors)
      .danger_accept_invalid_hostnames(options.ignore_tls_errors)
      .use_preconfigured_tls(tls_config);

    Retcher { 
      client: client.build().unwrap(), 
      config: RetcherConfig {
        browser: options.browser,
      }
    }
  }

  fn parse_url(&self, url: String) -> Result<Url, ErrorType> {
    let url = Url::parse(&url);

    if url.is_err() {
      return Err(ErrorType::UrlParseError);
    }
    let url = url.unwrap();

    if url.host_str().is_none() {
      return Err(ErrorType::UrlParseError);
    }

    let protocol = url.scheme();

    return match protocol {
      "http" => Ok(url),
      "https" => Ok(url),
      _ => Err(ErrorType::ProtocolError),
    };
  }

  pub fn builder() -> RetcherBuilder {
    RetcherBuilder::default()
  }

  pub async fn get(&self, url: String, options: Option<RequestOptions>) -> Result<Response, ErrorType> {
    let url = self.parse_url(url);

    if url.is_err() {
      return Err(url.err().unwrap());
    }

    let url = url.unwrap();

    let headers = HttpHeaders::get_builder()
      .with_browser(self.config.browser)
      .with_host(url.host_str().unwrap().to_string())
      .with_https(url.scheme() == "https")
      .with_custom_headers(options.unwrap_or_default().headers)
      .build();

    let request = self.client.get(url)
      .headers(headers.into());

    let response: Result<Response, reqwest::Error> = request.send().await;

    if response.is_err() {
      println!("Error: {:?}", response.err().unwrap());
      return Err(ErrorType::RequestError);
    }
    
    Ok(response.unwrap())
  }
}
