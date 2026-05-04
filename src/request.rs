use std::{collections::HashMap, fmt};

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct ScreenshotRequest {
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub width: i64,
    #[serde(default)]
    pub height: i64,
    #[serde(default)]
    pub format: String,
    #[serde(default)]
    pub quality: i64,
    #[serde(default)]
    pub wait_time: i64,
    #[serde(default)]
    pub wait_for: String,
    #[serde(default)]
    pub full_page: bool,
    #[serde(default)]
    pub headers: HashMap<String, String>,
    #[serde(default)]
    pub user_agent: String,
    #[serde(default)]
    pub clip: Option<ClipRect>,
    #[serde(default)]
    pub device_scale: f64,
    #[serde(default)]
    pub mobile: bool,
    #[serde(default)]
    pub landscape: bool,
    #[serde(default)]
    pub timeout: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ScreenshotQuery {
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub width: i64,
    #[serde(default)]
    pub height: i64,
    #[serde(default)]
    pub format: String,
    #[serde(default)]
    pub quality: i64,
    #[serde(default)]
    pub wait_time: i64,
    #[serde(default)]
    pub wait_for: String,
    #[serde(default)]
    pub full_page: bool,
    #[serde(default)]
    pub headers: Option<String>,
    #[serde(default)]
    pub user_agent: String,
    #[serde(default)]
    pub clip: Option<String>,
    #[serde(default)]
    pub device_scale: f64,
    #[serde(default)]
    pub mobile: bool,
    #[serde(default)]
    pub landscape: bool,
    #[serde(default)]
    pub timeout: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ClipRect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Clone)]
pub struct ValidationError {
    field: &'static str,
    message: &'static str,
}

impl ValidationError {
    fn new(field: &'static str, message: &'static str) -> Self {
        Self { field, message }
    }
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.field, self.message)
    }
}

impl std::error::Error for ValidationError {}

impl ScreenshotRequest {
    pub fn from_query(query: ScreenshotQuery) -> Result<Self, String> {
        let mut request = Self {
            url: query.url,
            width: query.width,
            height: query.height,
            format: query.format,
            quality: query.quality,
            wait_time: query.wait_time,
            wait_for: query.wait_for,
            full_page: query.full_page,
            headers: HashMap::new(),
            user_agent: query.user_agent,
            clip: None,
            device_scale: query.device_scale,
            mobile: query.mobile,
            landscape: query.landscape,
            timeout: query.timeout,
        };

        if let Some(raw) = query.headers.filter(|raw| !raw.trim().is_empty()) {
            request.headers =
                serde_json::from_str(&raw).map_err(|_| "invalid headers JSON".to_string())?;
        }

        if let Some(raw) = query.clip.filter(|raw| !raw.trim().is_empty()) {
            request.clip =
                Some(serde_json::from_str(&raw).map_err(|_| "invalid clip JSON".to_string())?);
        }

        Ok(request)
    }

    pub fn apply_defaults(&mut self) {
        if self.width <= 0 {
            self.width = 1920;
        }
        if self.height <= 0 {
            self.height = 1080;
        }
        if self.format.trim().is_empty() {
            self.format = "png".to_string();
        }
        self.format = self.format.trim().to_ascii_lowercase();
        if self.quality <= 0 || self.quality > 100 {
            self.quality = 90;
        }
        if self.wait_time < 0 {
            self.wait_time = 0;
        }
        if self.device_scale <= 0.0 {
            self.device_scale = 1.0;
        }
        if self.timeout <= 0 {
            self.timeout = 30;
        }
        if self.timeout > 120 {
            self.timeout = 120;
        }
    }

    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.url.trim().is_empty() {
            return Err(ValidationError::new("url", "is required"));
        }

        if !matches!(self.format.as_str(), "png" | "jpeg" | "jpg" | "webp") {
            return Err(ValidationError::new(
                "format",
                "must be png, jpeg, jpg, or webp",
            ));
        }

        if self.width > 4096 || self.width < 100 {
            return Err(ValidationError::new(
                "width",
                "must be between 100 and 4096",
            ));
        }

        if self.height > 10000 || self.height < 100 {
            return Err(ValidationError::new(
                "height",
                "must be between 100 and 10000",
            ));
        }

        if let Some(clip) = &self.clip {
            if clip.width <= 0.0 || clip.height <= 0.0 {
                return Err(ValidationError::new(
                    "clip",
                    "width and height must be greater than 0",
                ));
            }
        }

        Ok(())
    }

    pub fn content_type(&self) -> &'static str {
        match self.format.as_str() {
            "jpeg" | "jpg" => "image/jpeg",
            "webp" => "image/webp",
            _ => "image/png",
        }
    }

    pub fn filename(&self) -> &'static str {
        match self.format.as_str() {
            "jpeg" | "jpg" => "screenshot.jpeg",
            "webp" => "screenshot.webp",
            _ => "screenshot.png",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn applies_defaults_like_the_go_service() {
        let mut request = ScreenshotRequest {
            url: "https://example.com".to_string(),
            width: 0,
            height: 0,
            format: String::new(),
            quality: 0,
            wait_time: -10,
            wait_for: String::new(),
            full_page: false,
            headers: HashMap::new(),
            user_agent: String::new(),
            clip: None,
            device_scale: 0.0,
            mobile: false,
            landscape: false,
            timeout: 999,
        };

        request.apply_defaults();

        assert_eq!(request.width, 1920);
        assert_eq!(request.height, 1080);
        assert_eq!(request.format, "png");
        assert_eq!(request.quality, 90);
        assert_eq!(request.wait_time, 0);
        assert_eq!(request.device_scale, 1.0);
        assert_eq!(request.timeout, 120);
    }

    #[test]
    fn parses_headers_and_clip_from_query_json() {
        let request = ScreenshotRequest::from_query(ScreenshotQuery {
            url: "https://example.com".to_string(),
            width: 1280,
            height: 720,
            format: "jpg".to_string(),
            quality: 80,
            wait_time: 0,
            wait_for: String::new(),
            full_page: false,
            headers: Some(r#"{"Authorization":"Bearer token"}"#.to_string()),
            user_agent: String::new(),
            clip: Some(r#"{"x":1,"y":2,"width":300,"height":200}"#.to_string()),
            device_scale: 1.0,
            mobile: false,
            landscape: false,
            timeout: 30,
        })
        .expect("query should parse");

        assert_eq!(request.headers["Authorization"], "Bearer token");
        assert_eq!(request.clip.expect("clip").width, 300.0);
    }
}
