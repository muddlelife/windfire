extern crate core;

mod httpclient;
mod utils;

use crate::httpclient::{PrintInfo, create_http_client, send_request};
use crate::utils::{add_path, queue_to_csv, queue_to_json, read_file};
use clap::Parser;
use crossbeam::queue::SegQueue;
use futures::StreamExt;
use once_cell::sync::Lazy;
use serde::Deserialize;
use std::sync::Arc;

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
    version = "0.0.3",
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
    #[arg(short = 'c', long, default_value = "0")]
    status_code: String,

    /// Designated path scan
    #[arg(short = 'p', long, default_value = "")]
    path: String,

    /// Supported Proxy socks5, http, and https, Example: -x socks5://127.0.0.1:1080
    #[arg(short = 'x', long)]
    proxy: Option<String>,

    /// Output can be a CSV or JSON file. Example: -o result.csv or -o result.json
    #[arg(short = 'o', long)]
    output: Option<String>,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let status_code = args
        .status_code
        .split(',')
        .map(|s| s.parse::<u16>().unwrap())
        .collect::<Vec<u16>>();

    let path = args.path;
    let proxy = args.proxy;
    let seg_queue: Arc<SegQueue<PrintInfo>> = Arc::new(SegQueue::new());

    // 单个URL
    if let Some(url) = args.url {
        // url
        let client = create_http_client(args.timeout, proxy);

        // 处理url后缀并加上path
        let new_url = add_path(&url, &path);

        // 进行请求
        let result = send_request(client, &new_url, status_code, &seg_queue).await;
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
        // file
        let urls = read_file(&file);
        match urls {
            Ok(urls) => {
                let client = create_http_client(args.timeout, proxy);
                let max_concurrency = args.thread;
                futures::stream::iter(urls)
                    .map(|url| {
                        let client = client.clone();
                        let value = status_code.clone();
                        let seg_queue = seg_queue.clone();

                        // 处理url后缀并加上path
                        let new_url = add_path(&url, &path);

                        async move {
                            let result = send_request(client, &new_url, value, &seg_queue).await;
                            match result {
                                Ok(msg) if !msg.is_empty() => println!("{}", msg),
                                _ => {}
                            }
                        }
                    })
                    .buffer_unordered(max_concurrency)
                    .collect::<Vec<()>>()
                    .await;

                if let Some(output) = args.output {
                    // 修改下两种格式，如果后缀为csv则保存为csv格式，如果为json则保存为json格式
                    if output.ends_with(".csv") {
                        queue_to_csv(&seg_queue, output.as_str()).ok();
                    } else if output.ends_with(".json") {
                        queue_to_json(&seg_queue, output.as_str()).ok();
                    } else {
                        println!("The saved results are only available in CSV and JSON formats")
                    }
                }
            }
            Err(e) => {
                println!("{}", e)
            }
        }
    } else {
        println!("Please enter a url or file path");
    }
}
