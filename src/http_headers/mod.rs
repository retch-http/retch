use std::{collections::HashMap, str::FromStr};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use super::Browser;

mod statics;

pub struct HttpHeaders {
    context: HttpHeadersBuilder,
}

impl HttpHeaders {
    pub fn new(options: &HttpHeadersBuilder) -> HttpHeaders {
        HttpHeaders {
            context: options.clone(),
        }
    }

    pub fn get_builder() -> HttpHeadersBuilder {
        HttpHeadersBuilder::default()
    }
}

impl Into<HeaderMap> for HttpHeaders {
    fn into(self) -> HeaderMap {
        let mut headers = HeaderMap::new();

        let header_values = match self.context.browser {
            Some(Browser::Chrome) => statics::CHROME_HEADERS,
            None => &[]
        };

        // TODO: don't use HTTP2 headers for HTTP1.1
        for (name, value) in header_values {
            let value = match *name {
                "Host" => String::from(self.context.host.as_str()),
                _ => String::from(*value)
            };

            headers.append(
                HeaderName::from_str(name).unwrap(), 
                HeaderValue::from_str(value.as_str()).unwrap()
            );
        }

        headers
    }
}

#[derive(Default, Clone)]
pub struct HttpHeadersBuilder {
    host: String,
    browser: Option<Browser>,
    https: bool,
    custom_headers: HashMap<String, String>,
}

impl HttpHeadersBuilder {
    // TODO: Enforce `with_host` to be called before `build`
    pub fn with_host (&mut self, host: String) -> &mut Self {
        self.host = host;
        self
    }

    pub fn with_browser (&mut self, browser: Option<Browser>) -> &mut Self {
        self.browser = browser;
        self
    }

    pub fn with_https (&mut self, https: bool) -> &mut Self {
        self.https = https;
        self
    }

    pub fn with_custom_headers (&mut self, custom_headers: HashMap<String, String>) -> &mut Self {
        self.custom_headers = custom_headers;
        self
    }

    pub fn build(&self) -> HttpHeaders {
        HttpHeaders::new(self)
    }
}


