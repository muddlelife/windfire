use crate::cli::cli_options::ScanMode;
use crate::fingerprint::matcher::FingerprintMatcher;
use crate::http::request;
use crate::output::SaveInfo;
use reqwest::Client;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct ScanResult {
    pub url: String,
    pub status: u16,
    pub title: String,
    pub server: String,
    pub content_length: usize,
    pub jump_url: String,
    pub cms: Vec<String>,
}

pub async fn scan_one(
    client: &Client,
    url: &str,
    fingerprints: &FingerprintMatcher,
    status_code: &[u16],
    mode: ScanMode,
) -> Result<SaveInfo, Box<dyn std::error::Error + Send + Sync>> {
    let resp = request::fetch(client, url, status_code, mode == ScanMode::Full).await?;
    let matched = if mode == ScanMode::Full {
        fingerprints.match_response(&resp)
    } else {
        Vec::new()
    };

    Ok(SaveInfo {
        url: resp.url,
        status: resp.status,
        title: resp.title,
        server: resp.server,
        content_length: resp.content_length,
        jump_url: resp.jump_url,
        cms: matched.join("||"),
    })
}
