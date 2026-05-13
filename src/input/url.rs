use crate::cli::cli_options::Args;
use crate::input::cidr::{expand_cidr, is_cidr};
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub fn build_urls(args: &Args) -> Result<Vec<String>, Box<dyn Error>> {
    let mut urls = Vec::new();

    // 单个url
    if let Some(url) = &args.url {
        // 开头有http协议
        if url.starts_with("http://") || url.starts_with("https://") {
            urls.push(url.clone());
        } else if is_cidr(url) {
            // 针对ip地址和网段
            let ip_vec: Vec<String> = expand_cidr(url);
            for ip in ip_vec {
                urls.push(format!("http://{}", ip));
                urls.push(format!("https://{}", ip));
            }
        } else {
            // 针对ip地址加端口以及域名加端口，还得判断下是否为80端口
            urls.extend(expand_ip_or_domain(url))
        };
    }

    // 文件url
    if let Some(file) = &args.file {
        // let mut file_urls
        let file_urls = read_urls_from_file(file)?;
        urls.extend(file_urls);
    }

    // 添加path
    if let Some(path) = &args.path {
        urls = normalize_urls(urls, path);
    }
    // 去重复
    urls.sort();
    urls.dedup();
    Ok(urls)
}

fn has_http_scheme(input: &str) -> bool {
    input.starts_with("http://") || input.starts_with("https://")
}

// 读取文件
fn read_urls_from_file(path: &str) -> Result<Vec<String>, Box<dyn Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let mut result = Vec::new();

    for line in reader.lines() {
        let line = line?;
        let line = line.trim();

        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let expanded = expand_line(line);
        result.extend(expanded);
    }

    Ok(result)
}

// 解析http
fn expand_line(line: &str) -> Vec<String> {
    if has_http_scheme(line) {
        return vec![line.to_string()];
    }

    if is_cidr(line) {
        let mut ip_url = Vec::new();
        let ip_vec = expand_cidr(line);

        // 包含 http 和 https
        for ip in ip_vec {
            ip_url.extend(expand_ip_or_domain(&ip));
        }
        return ip_url;
    }

    expand_ip_or_domain(line)
}

// 协议规则
fn expand_ip_or_domain(input: &str) -> Vec<String> {
    if has_http_scheme(input) {
        return vec![input.to_string()];
    }

    if let Some((_, port)) = input.rsplit_once(':') {
        match port {
            "443" => vec![format!("https://{}", input)],
            "80" => vec![format!("http://{}", input)],
            _ => vec![format!("http://{}", input), format!("https://{}", input)],
        }
    } else {
        vec![format!("http://{}", input), format!("https://{}", input)]
    }
}

fn normalize_urls(urls: Vec<String>, path: &str) -> Vec<String> {
    let mut result = Vec::with_capacity(urls.len());

    let path = path.trim_start_matches('/');

    for url in urls {
        if url.ends_with('/') {
            result.push(format!("{}{}", url, path));
        } else {
            result.push(format!("{}/{}", url, path));
        }
    }
    result
}
