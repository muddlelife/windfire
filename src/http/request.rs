use crate::http::response::ResponseInfo;
use crate::utils::favicon::fetch_favicon_hash;
use reqwest::Client;

pub async fn fetch(client: &Client, original_url: &str) -> Result<ResponseInfo, reqwest::Error> {
    let resp = client.get(original_url).send().await?;
    let favicon_hash = fetch_favicon_hash(client, original_url).await;

    ResponseInfo::from_response(original_url, resp, favicon_hash).await
}
