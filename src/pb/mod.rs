pub mod abi;

use serde::Serialize;

#[derive(Serialize)]
pub struct ApiResponse<T> {
    success: bool,
    message: String,
    data: Option<T>,
}

impl<T> ApiResponse<T> {
    pub fn success(message: &str, data: T) -> Self {
        Self {
            success: true,
            message: message.to_string(),
            data: Some(data),
        }
    }

    pub fn error(message: &str) -> Self {
        Self {
            success: false,
            message: message.to_string(),
            data: None,
        }
    }
}

impl From<abi::ShortUrlResponse> for ApiResponse<abi::ShortUrlResponse> {
    fn from(res: abi::ShortUrlResponse) -> Self {
        ApiResponse {
            success: true,
            message: "OK".to_string(),
            data: Some(res),
        }
    }
}

impl From<abi::GetUserUrlsResponse> for ApiResponse<abi::GetUserUrlsResponse> {
    fn from(res: abi::GetUserUrlsResponse) -> Self {
        ApiResponse {
            success: true,
            message: "OK".to_string(),
            data: Some(res),
        }
    }
}
