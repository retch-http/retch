use std::{str::FromStr, time::Duration};
use log::debug;
use reqwest::{Method, Response, Version};
use url::Url;

use crate::{http3::H3Engine, http_headers::HttpHeaders, tls, RequestOptions};
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
  pub(self) base_client: reqwest::Client,
  pub(self) h3_client: Option<reqwest::Client>,
  h3_engine: Option<H3Engine>,
  config: RetcherBuilder,
}

impl Default for Retcher {
  fn default() -> Self {
    RetcherBuilder::default().build()
  }
}

#[derive(Debug, Clone)]
pub struct RetcherBuilder {
  browser: Option<Browser>,
  ignore_tls_errors: bool,
  vanilla_fallback: bool,
  proxy_url: String,
  request_timeout: Duration,
  max_http_version: Version,
}

impl Default for RetcherBuilder {
  fn default() -> Self {
    RetcherBuilder {
      browser: None,
      ignore_tls_errors: false,
      vanilla_fallback: true,
      proxy_url: String::from_str("").unwrap(),
      request_timeout: Duration::from_secs(30),
      max_http_version: Version::HTTP_2,
    }
  }
}

impl RetcherBuilder {
  pub fn with_browser(mut self, browser: Browser) -> Self {
    self.browser = Some(browser);
    self
  }

  /// If set to true, the client will ignore TLS-related errors.
  pub fn with_ignore_tls_errors(mut self, ignore_tls_errors: bool) -> Self {
    self.ignore_tls_errors = ignore_tls_errors;
    self
  }

  /// If set to true, the client will fallback to vanilla requests if the impersonated browser encounters an error.
  pub fn with_fallback_to_vanilla(mut self, vanilla_fallback: bool) -> Self {
    self.vanilla_fallback = vanilla_fallback;
    self
  }
  
  /// Sets the proxy URL to use for requests.
  pub fn with_proxy(mut self, proxy_url: String) -> Self {
    self.proxy_url = proxy_url;
    self
  }

  /// Sets the default timeout for requests.
  pub fn with_default_timeout(mut self, timeout: Duration) -> Self {
    self.request_timeout = timeout;
    self
  }

  /// Enables HTTP/3 usage for requests.
  /// 
  /// Note that this is experimental and may not work as expected with all servers.
  pub fn with_http3(mut self) -> Self {
    self.max_http_version = Version::HTTP_3;
    self
  }
  
  /// Builds the `Retcher` instance.
  pub fn build(self) -> Retcher {
    Retcher::new(self)
  }
}

impl Retcher {
  pub fn builder() -> RetcherBuilder {
    RetcherBuilder::default()
  }

  fn new_reqwest_client(config: &RetcherBuilder) -> Result<reqwest::Client, reqwest::Error> {
    let mut client = reqwest::Client::builder();
    let mut tls_config_builder = tls::TlsConfig::builder();
    let mut tls_config_builder = tls_config_builder.with_browser(config.browser);

    if config.max_http_version == Version::HTTP_3 {
      tls_config_builder = tls_config_builder.with_http3();
    }

    let tls_config = tls_config_builder.build();
    
    client = client
      .danger_accept_invalid_certs(config.ignore_tls_errors)
      .danger_accept_invalid_hostnames(config.ignore_tls_errors)
      .use_preconfigured_tls(tls_config)
      .timeout(config.request_timeout);

    if config.max_http_version == Version::HTTP_3 {
      client = client.http3_prior_knowledge();
    }

    if config.proxy_url.len() > 0 {
      client = client.proxy(
        reqwest::Proxy::all(&config.proxy_url)
        .expect("The proxy_url option should be a valid URL.")
      );
    }

    client.build()
  }

  /// Creates a new `Retcher` instance with the given `EngineOptions`.
  fn new(config: RetcherBuilder) -> Self {
    let mut h3_client: Option<reqwest::Client> = None;
    let mut base_client = Self::new_reqwest_client(&config).unwrap();

    if config.max_http_version == Version::HTTP_3 {
      h3_client = Some(base_client);
      base_client = Self::new_reqwest_client(&RetcherBuilder {
        max_http_version: Version::HTTP_2,
        ..config.clone()
      }).unwrap();
    }

    Retcher { 
      base_client, 
      h3_client,
      config,
      h3_engine: None,
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

  async fn should_use_h3(self: &mut Self, host: &String) -> bool {
    if self.config.max_http_version < Version::HTTP_3 {
      debug!("HTTP/3 is disabled, falling back to TCP-based requests.");
      return false;
    }

    if let None = &self.h3_engine {
      self.h3_engine = Some(H3Engine::init().await);
    }

    self.h3_engine.as_mut().unwrap().host_supports_h3(host).await
  }

  async fn make_request(&mut self, method: Method, url: String, body: Option<Vec<u8>>, options: Option<RequestOptions>) -> Result<Response, ErrorType> {
    let options = options.unwrap_or_default();

    let parsed_url = self.parse_url(url.clone())
      .expect("URL should be a valid URL");
    let host = parsed_url.host_str().unwrap().to_string();

    let h3 = options.http3_prior_knowledge || self.should_use_h3(&host).await;

    let headers = HttpHeaders::get_builder()
      .with_browser(&self.config.browser)
      .with_host(&host)
      .with_https(parsed_url.scheme() == "https")
      .with_custom_headers(&options.headers)
      .build();

    let client = if h3 {
      debug!("Using QUIC for request to {}", url);
      self.h3_client.as_ref().unwrap()
    } else {
      debug!("{} doesn't seem to have HTTP3 support", url);
      &self.base_client
    };

    let mut request = client
      .request(method.clone(), parsed_url)
      .headers(headers.into());

    if h3 {
      request = request.version(Version::HTTP_3);
    }

    if let Some(timeout) = options.timeout {
      request = request.timeout(timeout);
    }

    request = match body {
      Some(body) => request.body(body),
      None => request
    };

    let response = request.send().await;

    if response.is_err() {
      return Err(ErrorType::RequestError);
    }
    
    let response = response.unwrap();
    
    if !h3 {
      if let Some(h3_engine) = self.h3_engine.as_mut() {
        h3_engine.set_h3_support(&host, false);
  
        if let Some(alt_svc) = response.headers().get("Alt-Svc") {
          let alt_svc = alt_svc.to_str().unwrap();
          if alt_svc.contains("h3") {
            debug!("{} supports HTTP/3 (alt-svc header), adding to Alt-Svc cache", host);
            h3_engine.set_h3_support(&host, true);
          }
        }
      }
    }
    
    Ok(response)
  }

  pub async fn get(&mut self, url: String, options: Option<RequestOptions>) -> Result<Response, ErrorType> {
    self.make_request(Method::GET, url, None, options).await
  }

  pub async fn head(&mut self, url: String, options: Option<RequestOptions>) -> Result<Response, ErrorType> {
    self.make_request(Method::HEAD, url, None, options).await
  }
  
  pub async fn options(&mut self, url: String, options: Option<RequestOptions>) -> Result<Response, ErrorType> {
    self.make_request(Method::OPTIONS, url, None, options).await
  }

  pub async fn trace(&mut self, url: String, options: Option<RequestOptions>) -> Result<Response, ErrorType> {
    self.make_request(Method::TRACE, url, None, options).await
  }

  pub async fn delete(&mut self, url: String, options: Option<RequestOptions>) -> Result<Response, ErrorType> {
    self.make_request(Method::DELETE, url, None, options).await
  }

  pub async fn post(&mut self, url: String, body: Option<Vec<u8>>, options: Option<RequestOptions>) -> Result<Response, ErrorType> {
    self.make_request(Method::POST, url, body, options).await
  }

  pub async fn put(&mut self, url: String, body: Option<Vec<u8>>, options: Option<RequestOptions>) -> Result<Response, ErrorType> {
    self.make_request(Method::PUT, url, body, options).await
  }

  pub async fn patch(&mut self, url: String, body: Option<Vec<u8>>, options: Option<RequestOptions>) -> Result<Response, ErrorType> {
    self.make_request(Method::PATCH, url, body, options).await
  }

}