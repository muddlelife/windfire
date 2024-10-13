use lazy_static::lazy_static;
use regex::Regex;
use reqwest::Response;
use std::net::Ipv4Addr;
use std::str::FromStr;
use tokio::fs::File;
use crossbeam::queue::SegQueue;
use csv::WriterBuilder;
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncBufReadExt, BufReader};

lazy_static! {
    // 定义一个全局静态的正则表达式，用于匹配 <title> 标签的内容
    static ref TITLE_REGEX: Regex = Regex::new(r"(?i)<title>(.*?)</title>").unwrap();
}

pub(crate) async fn read_file(path: &str) -> Result<Vec<String>, tokio::io::Error> {
    let file = File::open(path).await?;
    let reader = BufReader::new(file);
    let mut urls = Vec::new();
    let mut lines = reader.lines();
    while let Some(line) = lines.next_line().await? {
        if line.starts_with("http://") || line.starts_with("https://") {
            urls.push(line);
            continue;
        } else {
            // 判断是否为CIDR
            let ip_vec = if is_cidr(&line) {
                cidr_to_ip_range(&line)
            } else {
                vec![line.clone()]
            };

            // 加上http和https头
            for ip in ip_vec {
                // 还要判断下端口，如果端口为443，则只https
                if line.contains(":") {
                    let port = line.split(":").collect::<Vec<_>>()[1];
                    if port == "443" {
                        urls.push(format!("https://{}", ip));
                        continue;
                    } else if port == "80" {
                        urls.push(format!("http://{}", ip));
                        continue;
                    }
                    else {
                        urls.push(format!("http://{}", ip));
                        urls.push(format!("https://{}", ip));
                    }
                }
                else {
                    urls.push(format!("http://{}", ip));
                    urls.push(format!("https://{}", ip));
                }
            }
        }
    }
    // 最后去重复
    urls.sort();
    urls.dedup();
    Ok(urls)
}

// 获取目标结果
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ScanInfo {
    pub url: String,
    pub status_code: u16,
    pub title: String,
    pub server: String,
    pub jump_url: String, // 跳转后的url
    pub content_length: usize,
}

// 根据响应获取响应结果
pub async fn get_format_info(response: Response, url: String) -> ScanInfo {
    let status_code = response.status().as_u16();
    let jump_url = response.url().to_string();

    // 获取server
    let headers = response.headers().get("Server");
    let server = match headers {
        Some(s) => s.to_owned().to_str().unwrap_or("").to_string(),
        None => "".to_string(),
    };

    let content = response.text().await.unwrap_or("".to_string());
    // 获取长度
    let content_length = content.len();
    let title = extract_title(&content).unwrap_or("".to_string());

    ScanInfo {
        url,
        status_code,
        title,
        content_length,
        server,
        jump_url,
    }
}

// 提取 <title> 内容的函数
fn extract_title(html: &str) -> Option<String> {
    if let Some(caps) = TITLE_REGEX.captures(html) {
        Some(caps[1].trim().to_string())
    } else {
        None
    }
}

// 判断line是不是 CIDR格式，如果是，返回true，否则返回false
pub fn is_cidr(line: &str) -> bool {
    // 匹配 CIDR 格式，如 192.168.0.0/24
    let re = Regex::new(r"^(\d{1,3}\.){3}\d{1,3}/\d{1,2}$").unwrap();
    re.is_match(line)
}

// 将CIDR格式转换为ip地址，用vec 返回
pub fn cidr_to_ip_range(cidr: &str) -> Vec<String> {
    // 分割 CIDR 格式，提取 IP 地址和前缀长度
    let mut parts = cidr.split('/');
    let base_ip = parts.next().unwrap();
    let prefix_len: u32 = parts.next().unwrap().parse().unwrap();

    // 将基础 IP 地址解析为 Ipv4Addr
    let base_ip: Ipv4Addr = Ipv4Addr::from_str(base_ip).unwrap();

    // 将 Ipv4Addr 转换为 u32，便于后续操作
    let base_ip_u32: u32 = u32::from(base_ip);

    // 计算掩码
    let mask: u32 = !((1 << (32 - prefix_len)) - 1);

    // 网络地址，即 base_ip_u32 & mask
    let network_ip_u32 = base_ip_u32 & mask;

    // 广播地址，即 network_ip_u32 | !mask
    let broadcast_ip_u32 = network_ip_u32 | !mask;

    // 生成 IP 地址列表
    let mut ip_list = Vec::new();
    for ip_u32 in network_ip_u32..=broadcast_ip_u32 {
        let ip = Ipv4Addr::from(ip_u32);
        ip_list.push(ip.to_string());
    }

    ip_list
}

// 将结果转为csv表格
pub fn queue_to_csv(scan_info_queue: &SegQueue<ScanInfo>, path: &str)  -> Result<(), Box<dyn std::error::Error>>{
    let mut wtr = WriterBuilder::new().from_path(path)?;
    while let Some(info) = scan_info_queue.pop() {
        wtr.serialize(info)?;
    }
    Ok(())
}