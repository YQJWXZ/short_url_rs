use crate::pb::abi;
use serde::{Deserialize, Serialize};

// 为 Protobuf 类型实现 SQLx FromRow
impl sqlx::FromRow<'_, sqlx::sqlite::SqliteRow> for abi::ShortUrl {
    fn from_row(row: &sqlx::sqlite::SqliteRow) -> Result<Self, sqlx::Error> {
        use sqlx::Row;
        Ok(abi::ShortUrl {
            id: row.get("id"),
            long_url: row.get("long_url"),
            short_code: row.get("short_code"),
            created_at: row.get("created_at"),
            expires_at: row.get("expires_at"),
            user_id: row.get("user_id"),
        })
    }
}

// 为 Protobuf 类型实现转换方法
impl abi::ShortUrl {
    pub fn to_response(&self, base_url: &str) -> abi::ShortUrlResponse {
        abi::ShortUrlResponse {
            id: self.id,
            long_url: self.long_url.clone(),
            short_code: self.short_code.clone(),
            short_url: format!("{}/{}", base_url, self.short_code),
            created_at: self.created_at.clone(),
            expires_at: self.expires_at.clone(),
        }
    }
}

// 为了兼容现有的 JSON API，实现 Serialize
impl Serialize for abi::ShortUrl {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("ShortUrl", 6)?;
        state.serialize_field("id", &self.id)?;
        state.serialize_field("long_url", &self.long_url)?;
        state.serialize_field("short_code", &self.short_code)?;
        state.serialize_field("created_at", &self.created_at)?;
        state.serialize_field("expires_at", &self.expires_at)?;
        state.serialize_field("user_id", &self.user_id)?;
        state.end()
    }
}

impl Serialize for abi::ShortUrlResponse {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("ShortUrlResponse", 6)?;
        state.serialize_field("id", &self.id)?;
        state.serialize_field("long_url", &self.long_url)?;
        state.serialize_field("short_code", &self.short_code)?;
        state.serialize_field("short_url", &self.short_url)?;
        state.serialize_field("created_at", &self.created_at)?;
        state.serialize_field("expires_at", &self.expires_at)?;
        state.end()
    }
}

// 为了兼容现有的 JSON API，实现 Deserialize
impl<'de> Deserialize<'de> for abi::CreateShortUrlRequest {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Helper {
            long_url: String,
            custom_code: Option<String>,
            timeout: Option<i64>,
            user_id: String,
        }

        let helper = Helper::deserialize(deserializer)?;
        Ok(abi::CreateShortUrlRequest {
            long_url: helper.long_url,
            custom_code: helper.custom_code,
            timeout: helper.timeout,
            user_id: helper.user_id,
        })
    }
}
