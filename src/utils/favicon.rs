use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use murmur3::murmur3_32;
use reqwest::{Client, Url};
use std::io::Cursor;

pub async fn fetch_favicon_hash(client: &Client, url: &str) -> Option<String> {
    let url = Url::parse(url).ok()?;
    url.join("favicon.ico").ok().map(|u| u.to_string());
    download_and_hash(client, url.as_ref()).await
}

// 计算hash，私有函数
async fn download_and_hash(client: &Client, url: &str) -> Option<String> {
    let resp = client.get(url).send().await.ok()?;

    if !resp.status().is_success() {
        return None;
    }

    let bytes = resp.bytes().await.ok()?;
    let base64 = STANDARD.encode(&bytes);

    let with_newlines: String = base64
        .as_bytes()
        .chunks(76) // 每 76 个字符分为一组
        .map(|chunk| String::from_utf8_lossy(chunk)) // 转为字符串
        .collect::<Vec<_>>() // 收集到 Vec<String>
        .join("\n"); // 在每组之间插入换行符

    // 然后对进行 mmnh编码
    let mut cursor = Cursor::new(with_newlines + "\n");
    match murmur3_32(&mut cursor, 0) {
        Ok(hash) => Some(hash.to_string()),
        Err(_) => None,
    }
}
