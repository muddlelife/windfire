use std::time::Duration;
use anyhow::{Error, Result};
use select::document::Document;
use select::predicate::Name;
use reqwest::{Client, header};

pub const USER_AGENT: &str ="Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:128.0) Gecko/20100101 Firefox/128.0";

// 创建http客户端
pub fn create_http_client(timeout: usize) -> Client {
    let mut headers = header::HeaderMap::new();
    headers.insert(
        header::USER_AGENT,
        header::HeaderValue::from_static(
            USER_AGENT
        ),
    );
    headers.insert(
        header::ACCEPT,
        header::HeaderValue::from_static(
            "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8",
        ),
    );
    headers.insert(
        header::CACHE_CONTROL,
        header::HeaderValue::from_static("max-age=0"),
    );
    headers.insert(header::DNT, header::HeaderValue::from_static("1"));
    headers.insert(
        header::UPGRADE_INSECURE_REQUESTS,
        header::HeaderValue::from_static("1"),
    );
    headers.insert(
        header::CONNECTION,
        header::HeaderValue::from_static("close"),
    );
    headers.insert(
        header::ACCEPT_LANGUAGE,
        header::HeaderValue::from_static("en-US,en;q=0.9,zh-CN;q=0.8,zh;q=0.7"),
    );

    Client::builder()
        .danger_accept_invalid_certs(true) // 忽略证书错误
        .default_headers(headers)
        .timeout(Duration::from_secs(timeout as u64))
        .build().expect("httpclient create failed!")
}


pub async fn send_request(client: Client,url: &str) -> Result<String,Error>{
    let response = client.get(url).send().await?;
    // 获取状态码
    let code = response.status();
    // 获取页面内容
    let content = response.text().await?;
    // 获取页面长度
    let length = content.len() % 1024;
    // 提取title
    let document = Document::from(content.clone().as_str());
    let title = document
        .find(Name("title"))
        .next()
        .map(|n| n.text())
        .unwrap_or("".to_string());

    Ok(format!("{} [{}] [{}] [{}kb]",url,code.as_u16(),title.trim(),length).into())
}


#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn test_send_request() {
        let client = create_http_client(10);
        let url = "https://www.baidu.com/";
        let result = send_request(client, url).await;
        println!("result:{}",result.unwrap());
    }
}