use url::Url;

// URL Validator
pub fn is_valid_url(url: &str) -> bool {
    Url::parse(url).is_ok()
}

// URL Normalizer
pub fn normalize_url(url: &str) -> String {
    if url.starts_with("http://") || url.starts_with("https://") {
        url.to_string()
    } else {
        format!("http://{}", url)
    }
}
