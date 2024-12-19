use std::{collections::HashMap, str::FromStr, time::Duration};
use reqwest::{Method, Response};
use url::Url;

use crate::{http_headers::HttpHeaders, tls};
use super::Browser;

#[derive(Debug, Clone)]
pub enum ErrorType {
  UrlParsingError,
  UrlMissingHostnameError,
  UrlProtocolError,
  ImpersonationError,
  RequestError,
  ResponseError,
}

/// Retcher is the main struct used to make (impersonated) requests.
/// 
/// It uses `reqwest::Client` to make requests and holds info about the impersonated browser.
pub struct Retcher {
  pub(self) client: reqwest::Client,
  config: RetcherConfig,
}

impl Default for Retcher {
  fn default() -> Self {
    RetcherConfig::default().build()
  }
}

#[derive(Debug, Clone)]
pub struct RetcherConfig {
  browser: Option<Browser>,
  ignore_tls_errors: bool,
  vanilla_fallback: bool,
  proxy_url: String,
  request_timeout: Duration,
}

impl Default for RetcherConfig {
  fn default() -> Self {
    RetcherConfig {
      browser: None,
      ignore_tls_errors: false,
      vanilla_fallback: true,
      proxy_url: String::from_str("").unwrap(),
      request_timeout: Duration::from_secs(30),
    }
  }
}

impl RetcherConfig {
  pub fn with_browser(mut self, browser: Browser) -> Self {
    self.browser = Some(browser);
    self
  }

  pub fn with_ignore_tls_errors(mut self, ignore_tls_errors: bool) -> Self {
    self.ignore_tls_errors = ignore_tls_errors;
    self
  }

  pub fn with_fallback_to_vanilla(mut self, vanilla_fallback: bool) -> Self {
    self.vanilla_fallback = vanilla_fallback;
    self
  }
  
  pub fn with_proxy(mut self, proxy_url: String) -> Self {
    self.proxy_url = proxy_url;
    self
  }

  pub fn with_timeout(mut self, timeout: Duration) -> Self {
    self.request_timeout = timeout;
    self
  }

  pub fn build(self) -> Retcher {
    Retcher::new(self)
  }
}

/// RequestOptions is a struct holding additional options for the fetch request.
#[derive(Debug, Clone)]
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
  fn new(config: RetcherConfig) -> Self {
    let mut client = reqwest::Client::builder();
    let tls_config = tls::TlsConfig::builder()
      .with_browser(config.browser)
      .build();
    
    client = client
      .danger_accept_invalid_certs(config.ignore_tls_errors)
      .danger_accept_invalid_hostnames(config.ignore_tls_errors)
      .use_preconfigured_tls(tls_config)
      .timeout(config.request_timeout);

    if config.proxy_url.len() > 0 {
      client = client.proxy(
        reqwest::Proxy::all(&config.proxy_url)
        .expect("The proxy_url option should be a valid URL.")
      );
    }

    Retcher { 
      client: client.build().unwrap(), 
      config
    }
  }

  fn parse_url(&self, url: String) -> Result<Url, ErrorType> {
    let url = Url::parse(&url);

    if url.is_err() {
      return Err(ErrorType::UrlParsingError);
    }
    let url = url.unwrap();

    if url.host_str().is_none() {
      return Err(ErrorType::UrlMissingHostnameError);
    }

    let protocol = url.scheme();

    return match protocol {
      "http" => Ok(url),
      "https" => Ok(url),
      _ => Err(ErrorType::UrlProtocolError),
    };
  }

  pub fn builder() -> RetcherConfig {
    RetcherConfig::default()
  }

  async fn make_request(&self, method: Method, url: String, body: Option<Vec<u8>>, options: Option<RequestOptions>) -> Result<Response, ErrorType> {
    let parsed_url = self
      .parse_url(url.clone())
      .expect("URL should be a valid URL");

    let headers = HttpHeaders::get_builder()
      .with_browser(self.config.browser)
      .with_host(parsed_url.host_str().unwrap().to_string())
      .with_https(parsed_url.scheme() == "https")
      .with_custom_headers(options.clone().unwrap_or_default().headers)
      .build();

    let request = self.client.request(method.clone(), parsed_url)
      .headers(headers.into());

    let request = match body {
      Some(body) => request.body(body),
      None => request
    };

    let response: Result<Response, reqwest::Error> = request.send().await;

    if response.is_err() {
      println!("{:#?}", response.err().unwrap());

      if !self.config.vanilla_fallback || self.config.browser.is_none() { 
        return Err(ErrorType::ImpersonationError)
      }

      return match Retcher::default().client.request(method, url).send().await {
        Ok(response) => Ok(response),
        Err(_) => Err(ErrorType::RequestError) // TODO: don't supress the error
      }
    }
    
    Ok(response.unwrap())
  }

  pub async fn get(&self, url: String, options: Option<RequestOptions>) -> Result<Response, ErrorType> {
    self.make_request(Method::GET, url, None, options).await
  }

  pub async fn head(&self, url: String, options: Option<RequestOptions>) -> Result<Response, ErrorType> {
    self.make_request(Method::HEAD, url, None, options).await
  }
  
  pub async fn options(&self, url: String, options: Option<RequestOptions>) -> Result<Response, ErrorType> {
    self.make_request(Method::OPTIONS, url, None, options).await
  }

  pub async fn trace(&self, url: String, options: Option<RequestOptions>) -> Result<Response, ErrorType> {
    self.make_request(Method::TRACE, url, None, options).await
  }

  pub async fn delete(&self, url: String, options: Option<RequestOptions>) -> Result<Response, ErrorType> {
    self.make_request(Method::DELETE, url, None, options).await
  }

  pub async fn post(&self, url: String, body: Option<Vec<u8>>, options: Option<RequestOptions>) -> Result<Response, ErrorType> {
    self.make_request(Method::POST, url, body, options).await
  }

  pub async fn put(&self, url: String, body: Option<Vec<u8>>, options: Option<RequestOptions>) -> Result<Response, ErrorType> {
    self.make_request(Method::PUT, url, body, options).await
  }

  pub async fn patch(&self, url: String, body: Option<Vec<u8>>, options: Option<RequestOptions>) -> Result<Response, ErrorType> {
    self.make_request(Method::PATCH, url, body, options).await
  }
}
