use crate::http::response::ResponseInfo;
use crate::utils::favicon::fetch_favicon_hash;
use reqwest::Client;

pub async fn fetch(
    client: &Client,
    original_url: &str,
    status_code: &[u16],
    with_favicon: bool,
) -> Result<ResponseInfo, Box<dyn std::error::Error + Send + Sync>> {
    let resp = client.get(original_url).send().await?;

    // 这块过滤状态码
    if status_code.contains(&resp.status().as_u16()) || status_code.is_empty() {
        let favicon_hash = if with_favicon {
            fetch_favicon_hash(client, original_url).await
        } else {
            None
        };
        ResponseInfo::from_response(original_url, resp, favicon_hash).await
    } else {
        Err("filtered by status code".into())
    }
}
