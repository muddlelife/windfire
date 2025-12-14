use reqwest::header::HeaderMap;
use reqwest::{Client, Proxy, header};
use std::time::Duration;

// 创建 http header头
fn create_http_header() -> HeaderMap {
    let mut headers = HeaderMap::new();

    headers.insert(
        header::USER_AGENT,
        header::HeaderValue::from_static(
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:128.0) Gecko/20100101 Firefox/128.0",
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
    // shiro
    headers.insert(
        header::COOKIE,
        header::HeaderValue::from_static("rememberMe=yyds"),
    );

    headers
}

// 创建 http 客户端
pub fn create_http_client(
    timeout: u64,
    proxy: Option<String>,
) -> Result<Client, Box<dyn std::error::Error>> {
    let mut builder = Client::builder()
        .timeout(Duration::from_secs(timeout))
        .danger_accept_invalid_certs(true)
        .default_headers(create_http_header());

    // 只有当 proxy 有值时才添加配置
    if let Some(proxy_url) = proxy {
        let p = Proxy::all(proxy_url)?;
        builder = builder.proxy(p);
    }
    Ok(builder.build()?)
}
