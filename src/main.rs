use crate::httpclient::{create_http_client, send_request, PrintInfo};
use crate::utils::{queue_to_csv, read_file};
use clap::Parser;
use crossbeam::queue::SegQueue;
use futures::future::join_all;
use once_cell::sync::Lazy;
use reqwest::Client;
use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::Semaphore;
use tokio::task;

mod httpclient;
mod utils;

#[derive(Debug, Deserialize)]
pub struct Finger {
    cms: String,
    method: String,
    location: String,
    keyword: Vec<String>,
}

// 使用 once_cell::Lazy 来延迟初始化静态变量
static FINGER_DATA: Lazy<Vec<Finger>> = Lazy::new(|| {
    let json_data = include_str!("finger.json"); // 使用 include_str! 将文件嵌入到程序中
    serde_json::from_str(json_data).expect("Failed to parse JSON") // 解析 JSON 数据
});

#[derive(Parser, Debug)]
#[command(
    version = "1.5.0",
    about = "An efficient and fast url survival detection tool",
    long_about = "Efficient URL activity tester written in Rust. Fast, batch, and lightweight"
)]
struct Args {
    /// Setting the number of threads
    #[arg(short, long, default_value = "50")]
    thread: usize,

    /// Enter an url
    #[arg(short = 'u', long)]
    url: Option<String>,

    /// Enter a file path
    #[arg(short = 'f', long)]
    file: Option<String>,

    /// The http request timeout
    #[arg(short = 's', long, default_value = "10")]
    timeout: usize,

    /// Display the specified status code
    #[arg(short = 'c', long, default_value = "200")]
    status_code: String,

    /// Designated path scan
    #[arg(short = 'p', long, default_value = "")]
    path: String,

    /// Supported Proxy socks5, http, and https, Example: -x socks5://127.0.0.1:1080
    #[arg(short = 'x', long)]
    proxy: Option<String>,

    /// Output is an csv document, Example: -o result.csv
    #[arg(short = 'o', long)]
    output: Option<String>,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let status_vec = args.status_code.split(",").collect::<Vec<_>>();

    let u16_vec = status_vec
        .into_iter()
        .map(|s| s.trim().parse::<u16>().ok().unwrap_or(200))
        .collect::<Vec<_>>();

    let path = args.path;
    let proxy = args.proxy;
    let seg_queue: Arc<SegQueue<PrintInfo>> = Arc::new(SegQueue::new());

    if let Some(url) = args.url {
        let client = create_http_client(args.timeout, proxy);
        let result = send_request(client, &url, u16_vec, &path, &seg_queue).await;
        match result {
            Ok(result) => {
                if result != "" {
                    println!("{}", result);
                }
            }
            Err(e) => {
                println!("{}", e);
            }
        }
    } else if let Some(file) = args.file {
        let urls = read_file(&*file).await;
        match urls {
            Ok(urls) => {
                let client = create_http_client(args.timeout, proxy);
                let semaphore = Arc::new(Semaphore::new(args.thread));
                let mut futures = Vec::new();
                for url in urls {
                    let semaphore = Arc::clone(&semaphore);
                    let client: Client = client.clone();
                    let u16_vec = u16_vec.clone();
                    let path: String = path.clone();
                    let seg_queue = Arc::clone(&seg_queue);
                    futures.push(task::spawn(async move {
                        let permit = semaphore.acquire().await.unwrap();
                        let result =
                            send_request(client, url.as_str(), u16_vec, &path, &seg_queue).await;
                        match result {
                            Ok(result) => {
                                if result != "" {
                                    println!("{}", result);
                                }
                            }
                            Err(_) => {}
                        }
                        drop(permit);
                    }));
                }
                join_all(futures).await;
                // 保存结果为CSV
                if let Some(output) = args.output {
                    queue_to_csv(&seg_queue, output.as_str()).ok();
                }
            }
            Err(e) => {
                println!("{}", e)
            }
        }
    }
}
