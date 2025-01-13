use std::{str::FromStr, time::Duration};
use log::debug;
use reqwest::{Method, Response, Version};
use url::Url;

use crate::{http3::H3Engine, http_headers::HttpHeaders, tls, request::RequestOptions, emulation::Browser};

/// Error types that can be returned by the `Retcher` struct.
/// 
/// The `ErrorType` enum is used to represent the different types of errors that can occur when making requests.
/// The `RequestError` variant is used to wrap the `reqwest::Error` type.
#[derive(Debug)]
pub enum ErrorType {
  /// The URL couldn't be parsed.
  UrlParsingError,
  /// The URL is missing the hostname.
  UrlMissingHostnameError,
  /// The URL uses an unsupported protocol.
  UrlProtocolError,
  /// The request was made with `http3_prior_knowledge`, but HTTP/3 usage wasn't enabled.
  Http3Disabled,
  /// `reqwest::Error` variant. See the nested error for more details.
  RequestError(reqwest::Error),
}

/// Retcher is the main struct used to make (impersonated) requests.
/// 
/// It uses `reqwest::Client` to make requests and holds info about the impersonated browser.
/// 
/// To create a new `Retcher` instance, use the [`Retcher::builder()`](RetcherBuilder) method.
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

/// Customizes the behavior of the `Retcher` struct when following redirects.
/// 
/// The `RedirectBehavior` enum is used to specify how the client should handle redirects.
#[derive(Debug, Clone)]
pub enum RedirectBehavior {
  /// Follow up to `usize` redirects.
  /// 
  /// If the number of redirects is exceeded, the client will return an error.
  FollowRedirect(usize),
  /// Don't follow any redirects.
  /// 
  /// The client will return the response for the first request, even with the `3xx` status code.
  ManualRedirect,
}

/// A builder struct used to create a new `Retcher` instance.
/// 
/// The builder allows setting the browser to impersonate, ignoring TLS errors, setting a proxy, and other options.
/// 
/// ### Example
/// ```rust
/// let retcher = Retcher::builder()
///   .with_browser(Browser::Firefox)
///   .with_ignore_tls_errors(true)
///   .with_proxy("http://localhost:8080".to_string())
///   .with_default_timeout(Duration::from_secs(10))
///   .with_http3()
///   .build();
/// 
/// let response = retcher.get("https://example.com".to_string(), None).await;
/// ```
#[derive(Debug, Clone)]
pub struct RetcherBuilder {
  browser: Option<Browser>,
  ignore_tls_errors: bool,
  vanilla_fallback: bool,
  proxy_url: String,
  request_timeout: Duration,
  max_http_version: Version,
  redirect: RedirectBehavior,
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
      redirect: RedirectBehavior::FollowRedirect(10),
    }
  }
}

impl RetcherBuilder {
  /// Sets the browser to impersonate.
  /// 
  /// The `Browser` enum is used to set the HTTP headers, TLS behaviour and other markers to impersonate a specific browser.
  /// 
  /// If not used, the client will use the default `reqwest` fingerprints.
  pub fn with_browser(mut self, browser: Browser) -> Self {
    self.browser = Some(browser);
    self
  }

  /// If set to true, the client will ignore TLS-related errors.
  pub fn with_ignore_tls_errors(mut self, ignore_tls_errors: bool) -> Self {
    self.ignore_tls_errors = ignore_tls_errors;
    self
  }

  /// If set to `true`, the client will retry the request without impersonation 
  /// if the impersonated browser encounters an error.
  pub fn with_fallback_to_vanilla(mut self, vanilla_fallback: bool) -> Self {
    self.vanilla_fallback = vanilla_fallback;
    self
  }
  
  /// Sets the proxy URL to use for requests.
  /// 
  /// Note that this proxy will be used for all the requests 
  /// made by the built `Retcher` instance.
  pub fn with_proxy(mut self, proxy_url: String) -> Self {
    self.proxy_url = proxy_url;
    self
  }

  /// Sets the default timeout for requests.
  /// 
  /// This setting can be overridden when making the request by using the `RequestOptions` struct.
  pub fn with_default_timeout(mut self, timeout: Duration) -> Self {
    self.request_timeout = timeout;
    self
  }

  /// Enables HTTP/3 usage for requests.
  ///
  /// `retch` currently supports HTTP/3 negotiation via the HTTPS DNS record and the `Alt-Svc` header.
  /// To enforce HTTP/3 usage, use the `http3_prior_knowledge` option in the `RequestOptions` struct when
  /// making the request.
  ///
  /// Note that this feature is experimental and may not work as expected with all servers.
  pub fn with_http3(mut self) -> Self {
    self.max_http_version = Version::HTTP_3;
    self
  }

  /// Sets the desired redirect behavior.
  ///
  /// By default, the client will follow up to 10 redirects.
  /// By passing the `RedirectBehavior::ManualRedirect` option, the client will not follow any redirects
  /// (i.e. it will return the response for the first request, with the 3xx status code).
  pub fn with_redirect(mut self, behavior: RedirectBehavior) -> Self {
    self.redirect = behavior;
    self
  }
  
  /// Builds the [`Retcher`] instance.
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

    tls_config_builder = tls_config_builder.with_ignore_tls_errors(config.ignore_tls_errors);

    let tls_config = tls_config_builder.build();

    client = client
      .danger_accept_invalid_certs(config.ignore_tls_errors)
      .danger_accept_invalid_hostnames(config.ignore_tls_errors)
      .use_preconfigured_tls(tls_config)
      .cookie_store(true)
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

    match config.redirect {
      RedirectBehavior::FollowRedirect(max) => {
        client = client.redirect(reqwest::redirect::Policy::limited(max));
      },
      RedirectBehavior::ManualRedirect => {
        client = client.redirect(reqwest::redirect::Policy::none());
      },
    }

    client.build()
  }

  /// Creates a new `Retcher` instance based on the options stored in the `RetcherBuilder` instance.
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

    if options.http3_prior_knowledge && self.config.max_http_version < Version::HTTP_3 {
      return Err(ErrorType::Http3Disabled);
    }

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
      return Err(ErrorType::RequestError(response.err().unwrap()));
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

  /// Makes a `GET` request to the specified URL.
  /// 
  /// The `url` parameter should be a valid URL.
  /// Additional options like `headers`, `timeout` or HTTP/3 usage can be passed via the `RequestOptions` struct.
  /// 
  /// If the request is successful, the `reqwest::Response` struct is returned.
  pub async fn get(&mut self, url: String, options: Option<RequestOptions>) -> Result<Response, ErrorType> {
    self.make_request(Method::GET, url, None, options).await
  }

  /// Makes a `HEAD` request to the specified URL.
  /// 
  /// The `url` parameter should be a valid URL.
  /// Additional options like `headers`, `timeout` or HTTP/3 usage can be passed via the `RequestOptions` struct.
  /// 
  /// If the request is successful, the `reqwest::Response` struct is returned.
  pub async fn head(&mut self, url: String, options: Option<RequestOptions>) -> Result<Response, ErrorType> {
    self.make_request(Method::HEAD, url, None, options).await
  }

  /// Makes an OPTIONS request to the specified URL.
  /// 
  /// The `url` parameter should be a valid URL.
  /// Additional options like `headers`, `timeout` or HTTP/3 usage can be passed via the `RequestOptions` struct.
  /// 
  /// If the request is successful, the `reqwest::Response` struct is returned.
  pub async fn options(&mut self, url: String, options: Option<RequestOptions>) -> Result<Response, ErrorType> {
    self.make_request(Method::OPTIONS, url, None, options).await
  }

  /// Makes a `TRACE` request to the specified URL.
  /// 
  /// The `url` parameter should be a valid URL.
  /// Additional options like `headers`, `timeout` or HTTP/3 usage can be passed via the `RequestOptions` struct.
  /// 
  /// If the request is successful, the `reqwest::Response` struct is returned.
  pub async fn trace(&mut self, url: String, options: Option<RequestOptions>) -> Result<Response, ErrorType> {
    self.make_request(Method::TRACE, url, None, options).await
  }

  /// Makes a `DELETE` request to the specified URL.
  /// 
  /// The `url` parameter should be a valid URL.
  /// Additional options like `headers`, `timeout` or HTTP/3 usage can be passed via the `RequestOptions` struct.
  /// 
  /// If the request is successful, the `reqwest::Response` struct is returned.
  pub async fn delete(&mut self, url: String, options: Option<RequestOptions>) -> Result<Response, ErrorType> {
    self.make_request(Method::DELETE, url, None, options).await
  }

  /// Makes a `POST` request to the specified URL.
  /// 
  /// The `url` parameter should be a valid URL.
  /// Additional options like `headers`, `timeout` or HTTP/3 usage can be passed via the `RequestOptions` struct.
  /// 
  /// If the request is successful, the `reqwest::Response` struct is returned.
  pub async fn post(&mut self, url: String, body: Option<Vec<u8>>, options: Option<RequestOptions>) -> Result<Response, ErrorType> {
    self.make_request(Method::POST, url, body, options).await
  }

  /// Makes a `PUT` request to the specified URL.
  /// 
  /// The `url` parameter should be a valid URL.
  /// Additional options like `headers`, `timeout` or HTTP/3 usage can be passed via the `RequestOptions` struct.
  /// 
  /// If the request is successful, the `reqwest::Response` struct is returned.
  pub async fn put(&mut self, url: String, body: Option<Vec<u8>>, options: Option<RequestOptions>) -> Result<Response, ErrorType> {
    self.make_request(Method::PUT, url, body, options).await
  }

  /// Makes a `PATCH` request to the specified URL.
  /// 
  /// The `url` parameter should be a valid URL.
  /// Additional options like `headers`, `timeout` or HTTP/3 usage can be passed via the `RequestOptions` struct.
  /// 
  /// If the request is successful, the `reqwest::Response` struct is returned.
  pub async fn patch(&mut self, url: String, body: Option<Vec<u8>>, options: Option<RequestOptions>) -> Result<Response, ErrorType> {
    self.make_request(Method::PATCH, url, body, options).await
  }

}