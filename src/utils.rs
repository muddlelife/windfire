use crate::httpclient::PrintInfo;
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use core::str;
use crossbeam::queue::SegQueue;
use csv::WriterBuilder;
use lazy_static::lazy_static;
use murmur3::murmur3_32;
use regex::Regex;
use reqwest::{Client, Response};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::io::{Cursor, Write};
use std::net::Ipv4Addr;
use std::str::FromStr;
use url::Url;

lazy_static! {
    // 定义一个全局静态的正则表达式，用于匹配 <title> 标签的内容
    static ref TITLE_REGEX: Regex = Regex::new(r"(?i)<title>(.*?)</title>").unwrap();
}

// csv保存格式
#[derive(Debug, Serialize)]
struct SaveInfo {
    pub url: String,
    pub status_code: u16,
    pub title: String,
    pub server: String,
    pub jump_url: String, // 跳转后的url
    pub content_length: usize,
    pub cms: String,
}
impl SaveInfo {
    fn new(print_info: PrintInfo) -> Self {
        Self {
            url: print_info.url,
            status_code: print_info.status_code,
            title: print_info.title,
            server: print_info.server,
            jump_url: print_info.jump_url,
            content_length: print_info.content_length,
            cms: print_info.cms.join(" || "),
        }
    }
}

// 读文件
pub fn read_file(path: &str) -> Result<Vec<String>, Box<dyn Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut urls = Vec::new();

    // let mut lines = reader.lines();
    for line in reader.lines() {
        let line = line?;
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
                    } else {
                        urls.push(format!("http://{}", ip));
                        urls.push(format!("https://{}", ip));
                    }
                } else {
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
    pub body: String,
    pub header: String,
}

// 根据响应获取响应结果
pub async fn get_format_info(response: Response, url: &str) -> ScanInfo {
    let status_code = response.status().as_u16();
    let jump_url = response.url().to_string();

    // 获取server
    let headers = response.headers().get("Server");
    let server = match headers {
        Some(s) => s.to_owned().to_str().unwrap_or("").to_string(),
        None => "".to_string(),
    };

    let header = response
        .headers()
        .iter()
        .map(|(key, value)| format!("{}: {}", key, value.to_str().unwrap_or("")))
        .collect::<Vec<_>>()
        .join("\n");

    let body: String = response.text().await.unwrap_or("".to_string());
    // 获取长度
    let content_length = body.len();
    let title = extract_title(&body).unwrap_or("".to_string());

    ScanInfo {
        url: url.to_string(),
        status_code,
        title,
        content_length,
        server,
        jump_url,
        body,
        header,
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

// 解析基础url
pub fn get_favicon_url(base_url: &str) -> Result<String, Box<dyn std::error::Error>> {
    // 解析基础 URL
    let parsed_url = Url::parse(base_url)?;

    // 拼接 favicon.ico
    let favicon_url = parsed_url.join("favicon.ico")?;

    // 返回拼接后的 URL
    Ok(favicon_url.to_string())
}

// 计算 fofa hash
pub async fn get_fofa_iconhash(
    icon_url: String,
    client: Client,
) -> Result<String, Box<dyn std::error::Error>> {
    if icon_url == "".to_string() {
        return Ok("".to_string());
    }

    let resp = client.get(icon_url).send().await?;
    // 确保请求成功
    if resp.status().is_success() {
        let bytes = resp.bytes().await?;

        // 进行 Base64 编码
        let base64_str = STANDARD.encode(&bytes);

        // 每 76 个字符插入一个换行符
        let with_newlines: String = base64_str
            .as_bytes()
            .chunks(76) // 每 76 个字符分为一组
            .map(|chunk| String::from_utf8_lossy(chunk)) // 转为字符串
            .collect::<Vec<_>>() // 收集到 Vec<String>
            .join("\n"); // 在每组之间插入换行符

        // 然后对进行 mmnh编码
        let mut cursor = Cursor::new(with_newlines + "\n");
        let hash_u32 = murmur3_32(&mut cursor, 0)?;
        let hash_i32 = hash_u32 as i32;

        // 将哈希值转换为十六进制字符串
        Ok(format!("{}", hash_i32.to_string()))
    } else {
        Err("".into())
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
pub fn queue_to_csv(
    scan_info_queue: &SegQueue<PrintInfo>,
    path: &str,
) -> Result<(), Box<dyn Error>> {
    let mut file = File::create(path)?;

    file.write_all(b"\xEF\xBB\xBF")?;
    let mut wtr = WriterBuilder::new().from_writer(file);

    while let Some(info) = scan_info_queue.pop() {
        // 如果 cms 是一个 Vec<String>，那么我们可能需要将它转换成一个逗号分隔的字符串
        let new_info = SaveInfo::new(info);

        // 将数据序列化到 CSV 文件中
        match wtr.serialize(new_info) {
            Ok(_) => {}
            Err(e) => {
                println!("Error serializing data: {:?}", e);
            }
        };
    }
    Ok(())
}

// 保存为json格式
pub fn queue_to_json(
    scan_info_queue: &SegQueue<PrintInfo>,
    path: &str,
) -> Result<(), Box<dyn Error>> {
    let mut data = Vec::new();

    // 从队列中取出所有数据
    while let Some(info) = scan_info_queue.pop() {
        let save_info = SaveInfo::new(info);
        data.push(save_info);
    }

    // 序列化为 JSON 并写入文件
    let file = File::create(path)?;
    serde_json::to_writer_pretty(file, &data)?; // 使用 to_writer_pretty 可读性更好

    Ok(())
}

// url加路径
pub fn add_path(url: &str, path: &str) -> String {
    if path.is_empty() {
        url.to_string()
    } else {
        if url.ends_with("/") {
            format!("{}{}", url, path)
        } else {
            format!("{}/{}", url, path)
        }
    }
}
