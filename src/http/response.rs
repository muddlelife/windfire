use once_cell::sync::Lazy;
use regex::Regex;
use reqwest::Response;

static TITLE_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?is)<title[^>]*>(.*?)</title>").unwrap());
#[derive(Debug, Clone)]
pub struct ResponseInfo {
    pub url: String,
    pub status: u16,
    pub title: String,
    pub server: String,
    pub content_length: usize,
    pub headers: String,
    pub jump_url: String,
    pub body: String,
    pub favicon_hash: Option<String>,
}
impl ResponseInfo {
    pub async fn from_response(
        original_url: &str,
        resp: Response,
        favicon_hash: Option<String>,
    ) -> Result<Self, reqwest::Error> {
        let status = resp.status().as_u16();
        let url = original_url.to_string();
        let jump_url = resp.url().to_string();

        let server = resp
            .headers()
            .get("Server")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_string();

        let header = resp
            .headers()
            .iter()
            .map(|(key, value)| format!("{}: {}", key, value.to_str().unwrap_or("")))
            .collect::<Vec<_>>()
            .join("\n");

        let body = resp.text().await.unwrap_or_default();
        let title = extract_title(&body);
        let content_length = body.len();

        Ok(ResponseInfo {
            url,
            status,
            title,
            server,
            content_length,
            headers: header,
            jump_url,
            body,
            favicon_hash,
        })
    }
}

// 获取title
fn extract_title(body: &str) -> String {
    TITLE_REGEX
        .captures(body)
        .and_then(|cap| cap.get(1))
        .map(|m| m.as_str().trim().to_string())
        .unwrap_or_default()
}
