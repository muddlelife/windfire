// 主要做解析参数
use url::Url;

use clap::{ArgGroup, Parser};

#[derive(Parser, Debug)]
#[command(
    version = "0.0.4",
    about = "An efficient and fast url survival detection tool",
    long_about = "Efficient URL activity tester written in Rust. Fast, batch, and lightweight",
    group(
        ArgGroup::new("target")
            .required(true)
            .args(["url", "file"]),
    )
)]
pub struct Args {
    /// Setting the number of threads
    #[arg(short, long, default_value = "50",value_parser= validate_thread_range)]
    pub thread: usize,

    /// Enter a url
    #[arg(short = 'u', long)]
    pub url: Option<String>,

    /// Enter a file path
    #[arg(short = 'f', long)]
    pub file: Option<String>,

    /// The http request timeout
    #[arg(short = 's', long, default_value = "10",value_parser = clap::value_parser!(u64).range(1..=60))]
    pub timeout: u64, //改为 u64 以适配 Duration::from_secs

    /// Display the specified status code
    #[arg(short = 'c', long, value_delimiter = ',')]
    pub status_code: Vec<u16>, // 优化状态码参数

    /// Designated path scan
    #[arg(short = 'p', long, default_value = "")]
    pub path: Option<String>,

    /// Supported Proxy socks5, http, and https, Example: -x socks5://127.0.0.1:1080
    #[arg(short = 'x', long, value_parser = validate_proxy_url)]
    pub proxy: Option<String>,

    /// Output can be a CSV or JSON file. Example: -o result.csv or -o result.json
    #[arg(short = 'o', long, value_parser = validate_output_format)]
    pub output: Option<String>,
}

// 验证线程参数，不超过5000
fn validate_thread_range(s: &str) -> Result<usize, String> {
    s.parse()
        .map_err(|_| "Invalid number of threads".to_string())
        .and_then(|n| {
            if (1..=5000).contains(&n) {
                Ok(n)
            } else {
                Err("Thread count must be between 1 and 5000".to_string())
            }
        })
}
// 验证代理参数
fn validate_proxy_url(s: &str) -> Result<String, String> {
    let url = Url::parse(s).map_err(|_| format!("'{}' is not a valid URL format", s))?;

    match url.scheme() {
        "http" | "https" | "socks5" => Ok(s.to_string()),
        _ => Err(
            "Invalid proxy protocol. Supported protocols are http, https and socks5".to_string(),
        ),
    }
}

// 验证输出参数
fn validate_output_format(s: &str) -> Result<String, String> {
    if s.ends_with(".csv") || s.ends_with(".json") {
        Ok(s.to_string())
    } else {
        Err("Invalid output format".to_string())
    }
}
