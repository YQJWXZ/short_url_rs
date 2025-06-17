use url::Url;

// URL Validator
pub fn is_valid_url(url: &str) -> bool {
    // 检查URL是否包含双斜杠，确保协议正确
    if !url.contains("://") {
        return false;
    }

    // 使用url库进行解析验证
    if let Ok(parsed_url) = Url::parse(url) {
        // 确保URL有主机部分
        parsed_url.host().is_some()
    } else {
        false
    }
}

// URL Normalizer
pub fn normalize_url(url: &str) -> String {
    if url.starts_with("http://") || url.starts_with("https://") || url.contains("://") {
        url.to_string()
    } else {
        format!("http://{}", url)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_valid_url() {
        // Valid URLs
        assert!(is_valid_url("https://www.example.com"));
        assert!(is_valid_url("http://example.com"));
        assert!(is_valid_url(
            "https://example.com/path?query=value#fragment"
        ));
        assert!(is_valid_url("http://localhost:8080"));
        assert!(is_valid_url("https://user:pass@example.com"));

        // Invalid URLs
        assert!(!is_valid_url("not-a-url"));
        assert!(!is_valid_url("http://"));
        assert!(!is_valid_url("://example.com"));
        assert!(!is_valid_url("http:/example.com"));
        assert!(!is_valid_url(""));
    }

    #[test]
    fn test_normalize_url() {
        // URLs with scheme should remain unchanged
        assert_eq!(normalize_url("http://example.com"), "http://example.com");
        assert_eq!(normalize_url("https://example.com"), "https://example.com");
        assert_eq!(normalize_url("ftp://example.com"), "ftp://example.com"); // Non-http schemes preserved

        // URLs without scheme should get http:// prefix
        assert_eq!(normalize_url("example.com"), "http://example.com");
        assert_eq!(normalize_url("www.example.com"), "http://www.example.com");
        assert_eq!(normalize_url("example.com/path"), "http://example.com/path");
        assert_eq!(normalize_url("localhost:8080"), "http://localhost:8080");

        // Edge cases
        assert_eq!(normalize_url(""), "http://");
    }
}
