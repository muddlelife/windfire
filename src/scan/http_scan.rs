use crate::fingerprint::model::Fingerprint;
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
    fingerprints: &[Fingerprint],
    status_code: &[u16],
) -> Result<SaveInfo, Box<dyn std::error::Error + Send + Sync>> {
    let resp = request::fetch(client, url, status_code).await?;
    let mut matched: Vec<String> = Vec::new();

    for fp in fingerprints {
        if fp.matches(&resp) {
            matched.push(fp.cms.clone());
        }
    }
    matched.sort();
    matched.dedup();

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
