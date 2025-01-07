use encoding::Encoding;

/// Implements the BOM sniffing algorithm to detect the encoding of the response.
/// If the BOM sniffing algorithm fails, the function returns `None`.
/// 
/// See more details at https://encoding.spec.whatwg.org/#bom-sniff
fn bom_sniffing(bytes: &Vec<u8>) -> Option<encoding::EncodingRef> {
    if [0xEF, 0xBB, 0xBF].to_vec() == bytes[0..3].to_vec() {
        return Some(encoding::all::UTF_8);
    }

    if [0xFE, 0xFF].to_vec() == bytes[0..2].to_vec() {
        return Some(encoding::all::UTF_16BE);
    }

    if [0xFF, 0xFE].to_vec() == bytes[0..2].to_vec() {
        return Some(encoding::all::UTF_16LE);
    }

    None
}

/// A lazy implementation of the BOM sniffing algorithm, using `scraper` to parse the HTML and extract the encoding.
/// 
/// See more details at https://html.spec.whatwg.org/#prescan-a-byte-stream-to-determine-its-encoding
pub fn prescan_bytestream(bytes: &Vec<u8>) -> Option<encoding::EncodingRef> {
    let limit = std::cmp::min(1024, bytes.len());

    let ascii_body = encoding::all::ASCII.decode(&bytes[0..limit], encoding::DecoderTrap::Replace).unwrap();
    let dom = scraper::Html::parse_document(&ascii_body);

    let meta = dom.select(&scraper::Selector::parse("meta[charset]").unwrap()).next();

    if let Some(meta) = meta {
        let charset = meta.value().attr("charset").unwrap();
        return encoding::label::encoding_from_whatwg_label(charset);
    }

    let meta = dom.select(&scraper::Selector::parse("meta[http-equiv=content-type]").unwrap()).next();

    if let Some(meta) = meta {
        let content = meta.value().attr("content").unwrap();
        let content_type = ContentType::from(content);

        return match content_type {
            Ok(content_type) => content_type.into(),
            Err(_) => None,
        }
    }

    None
}

/// Converts a vector of bytes to a string using the provided encoding.
pub fn decode(bytes: &Vec<u8>, encoding_prior_knowledge: Option<encoding::EncodingRef>) -> String {
    let mut encoding: encoding::EncodingRef = encoding::all::UTF_8;

    if let Some(enc) = encoding_prior_knowledge {
        encoding = enc;
    } else if let Some(enc) = bom_sniffing(&bytes) {
        encoding = enc;
    } else if let Some(enc) = prescan_bytestream(&bytes) {
        encoding = enc;
    }

    return encoding.decode(&bytes, encoding::DecoderTrap::Strict).unwrap();
}

pub struct ContentType {
    charset: String,
}

impl ContentType {
    pub fn from(content_type: &str) -> Result<Self, ()> {
        let parts: Vec<&str> = content_type.split("charset=").collect();

        if parts.len() != 2 || parts[1].len() == 0 {
            return Err(());
        }

        Ok(ContentType {
            charset: String::from(parts[1]),
        })
    }
}

impl Into<Option<encoding::EncodingRef>> for ContentType {
    fn into(self) -> Option<encoding::EncodingRef> {
        encoding::label::encoding_from_whatwg_label(self.charset.as_str())
    }
}