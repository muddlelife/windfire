use lazy_static::lazy_static;
use regex::Regex;
use reqwest::Response;
use tokio::fs::File;
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
        urls.push(line);
    }
    Ok(urls)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_file() {
        let urls = read_file("url.txt").unwrap();
        println!("{:?}", urls)
    }
}

// 获取目标结果
pub struct ScanInfo {
    pub status_code: u16,
    pub title: String,
    pub content_length: usize,
}

// 根据响应获取响应结果
pub async fn get_format_info(response: Response) -> ScanInfo {
    let status_code = response.status().as_u16();
    let content = response.text().await.unwrap_or("".to_string());
    // 获取长度
    let content_length = content.len() % 1024;
    let title = extract_title(&content).unwrap_or("".to_string());

    ScanInfo {
        status_code,
        title,
        content_length,
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
