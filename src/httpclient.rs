use crate::{
    utils::{get_favicon_url, get_fofa_iconhash, get_format_info},
    FINGER_DATA,
};
use crossbeam::queue::SegQueue;
use reqwest::{header, Client};
use serde::Serialize;
use std::time::Duration;

pub const USER_AGENT: &str =
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:128.0) Gecko/20100101 Firefox/128.0";

#[derive(Debug, Serialize, Clone)]
pub struct PrintInfo {
    pub url: String,
    pub status_code: u16,
    pub title: String,
    pub server: String,
    pub jump_url: String,
    pub content_length: usize,
    pub cms: Vec<String>,
}

// 创建http客户端
pub fn create_http_client(timeout: usize, proxy: Option<String>) -> Client {
    let mut headers = header::HeaderMap::new();
    headers.insert(
        header::USER_AGENT,
        header::HeaderValue::from_static(USER_AGENT),
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

    match proxy {
        Some(proxy_url) => {
            Client::builder()
                .danger_accept_invalid_certs(true) // 忽略证书错误
                .default_headers(headers)
                .proxy(reqwest::Proxy::all(proxy_url).expect("proxy url error"))
                .timeout(Duration::from_secs(timeout as u64))
                .build()
                .expect("httpclient create failed!")
        }
        None => {
            Client::builder()
                .danger_accept_invalid_certs(true) // 忽略证书错误
                .default_headers(headers)
                .timeout(Duration::from_secs(timeout as u64))
                .build()
                .expect("httpclient create failed!")
        }
    }
}

pub async fn send_request(
    client: Client,
    url: &str,
    u16_vec: Vec<u16>,
    path: &str,
    seg_queue: &SegQueue<PrintInfo>,
) -> Result<String, reqwest::Error> {
    // 解析URL，如果path为空，则默认为/，如果有值，则加上，还需要处理url有没有/
    let url = if path.is_empty() {
        url.to_string() // 如果 path 为空，返回原始的 url
    } else {
        // 处理路径不为空的情况
        if url.ends_with("/") {
            format!("{}{}", url, path)
        } else {
            format!("{}/{}", url, path)
        }
    };

    let response = client.get(url.as_str()).send().await?;

    let scan_info = get_format_info(response, url);
    let scan_info = scan_info.await;

    let url = scan_info.url;
    let status_code = scan_info.status_code;
    let title = scan_info.title;
    let content_length = scan_info.content_length;
    let server = scan_info.server;
    let jump_url = scan_info.jump_url;
    let body = scan_info.body;
    let header = scan_info.header;

    // 处理url 变为 协议 + 域名 + 端口 + /favicon.ico
    let favicon_url = get_favicon_url(&url).unwrap_or_else(|_| "".to_string());

    // 获取 icon
    let icon_hash = get_fofa_iconhash(favicon_url, client)
        .await
        .unwrap_or_else(|_| "".to_string());
    let mut cms_list: Vec<String> = Vec::new();

    // 然后进行指纹匹配，如果能匹配到，则返回cms，遍历指纹
    for finger in &*FINGER_DATA {
        let method = finger.method.to_string();
        let location = finger.location.to_string();
        let keyword = finger.keyword.clone();

        if method == "keyword" {
            // 说明为关键词匹配
            if location == "body" {
                // 说明是body匹配,keyword 词组都在 body中
                let all_found = keyword.iter().all(|s| body.contains(s));
                if all_found {
                    // 说明匹配成功
                    cms_list.push(finger.cms.to_string());
                }
            } else if location == "header" {
                // 说明是header匹配
                let all_found = keyword.iter().all(|s| header.contains(s));
                if all_found {
                    // 说明匹配成功
                    cms_list.push(finger.cms.to_string());
                }
            }
        } else if method == "faviconhash" {
            // 说明是faviconhash匹配
            if icon_hash == finger.keyword[0] {
                // 说明匹配成功
                cms_list.push(finger.cms.to_string());
            }
        }
    }

    // 对 cms_list 进行去重
    cms_list.dedup();

    if u16_vec.contains(&status_code) {
        seg_queue.push(PrintInfo {
            url: url.to_string(),
            status_code,
            title: title.to_string(),
            server: server.to_string(),
            jump_url: jump_url.to_string(),
            content_length,
            cms: cms_list.clone(),
        });
        Ok(format!(
            "{} [{}] [{}] [{}] [{}] [{}] {:?}",
            url, status_code, title, server, jump_url, content_length, cms_list
        ))
    } else {
        Ok("".to_string())
    }
}
