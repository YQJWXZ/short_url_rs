use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};

pub fn generate_short_code() -> String {
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(6)
        .map(char::from)
        .collect()
}

pub fn generate_custom_code(length: usize) -> String {
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(length)
        .map(char::from)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_generate_short_code_length() {
        // Default short code should be 6 characters
        let code = generate_short_code();
        assert_eq!(code.len(), 6);
    }

    #[test]
    fn test_generate_short_code_alphanumeric() {
        // Short code should only contain alphanumeric characters
        let code = generate_short_code();
        assert!(code.chars().all(|c| c.is_ascii_alphanumeric()));
    }

    #[test]
    fn test_generate_short_code_uniqueness() {
        // Generate multiple codes and check they are different
        // This is a probabilistic test, but with very high probability of success
        let mut codes = HashSet::new();
        for _ in 0..100 {
            codes.insert(generate_short_code());
        }
        // With 100 random 6-character codes, we expect nearly all to be unique
        assert!(codes.len() > 95);
    }

    #[test]
    fn test_generate_custom_code_length() {
        // Test various lengths
        for length in [3, 5, 8, 10, 15] {
            let code = generate_custom_code(length);
            assert_eq!(code.len(), length);
        }
    }

    #[test]
    fn test_generate_custom_code_alphanumeric() {
        // Custom code should only contain alphanumeric characters
        let code = generate_custom_code(10);
        assert!(code.chars().all(|c| c.is_ascii_alphanumeric()));
    }

    #[test]
    fn test_generate_custom_code_zero_length() {
        // Zero length should return empty string
        let code = generate_custom_code(0);
        assert_eq!(code, "");
    }
}
