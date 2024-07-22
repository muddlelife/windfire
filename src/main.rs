use futures::future::join_all;
use std::sync::Arc;
use clap::Parser;
use reqwest::Client;
use tokio::task;
use tokio::sync::Semaphore;
use crate::httpclient::{send_request,create_http_client};
use crate::utils::read_file;

mod httpclient;
mod utils;

#[derive(Parser, Debug)]
#[command(
    version = "1.0.0",
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
}


#[tokio::main]
async fn main() {
    let args = Args::parse();

    if let Some(url) = args.url {
        let client = create_http_client(args.timeout);
        let result = send_request(client, &url).await;
        match result {
            Ok(result) => {
                println!("{}", result);
            },
            Err(e) => {
                println!("{}", e);
            }
        }
    }
    else if let Some(file) = args.file {
        let urls = read_file(&*file).await;
        match urls {
            Ok(urls) => {
                let client = httpclient::create_http_client(args.timeout);
                let semaphore = Arc::new(Semaphore::new(args.thread));
                let mut futures = Vec::new();
                for url in urls {
                    let semaphore = Arc::clone(&semaphore);
                    let client: Client = client.clone();
                    futures.push(task::spawn(async move {
                        let permit = semaphore.acquire().await.unwrap();
                        let result = send_request(client,url.as_str()).await;
                        match result {
                            Ok(result) => {
                                println!("{}", result);
                            },
                            Err(_) => {}
                        }
                        drop(permit);
                    }));
                }
                join_all(futures).await;
            }
            Err(e) => {
                println!("{}",e)
            }
        }
    }
}