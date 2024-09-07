use crate::httpclient::{create_http_client, send_request};
use crate::utils::read_file;
use clap::Parser;
use futures::future::join_all;
use reqwest::Client;
use std::sync::Arc;
use tokio::sync::Semaphore;
use tokio::task;

mod httpclient;
mod utils;

#[derive(Parser, Debug)]
#[command(
    version = "1.3.0",
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

    if let Some(url) = args.url {
        let client = create_http_client(args.timeout, proxy);
        let result = send_request(client, &url, u16_vec, &path).await;
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
                    let path = path.clone();
                    futures.push(task::spawn(async move {
                        let permit = semaphore.acquire().await.unwrap();
                        let result = send_request(client, url.as_str(), u16_vec, &path).await;
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
            }
            Err(e) => {
                println!("{}", e)
            }
        }
    }
}
