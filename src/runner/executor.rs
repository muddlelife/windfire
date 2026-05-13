use crate::cli::cli_options::ScanMode;
use crate::fingerprint::matcher::FingerprintMatcher;
use crate::output::SaveInfo;
use crate::scan::http_scan::scan_one;
use futures::StreamExt;
use reqwest::Client;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::mpsc;
use tokio::time::{Duration, interval};

pub async fn run(
    client: Client,
    urls: Vec<String>,
    fingerprint: Arc<FingerprintMatcher>,
    threads: usize,
    status_code: Arc<Vec<u16>>,
    mode: ScanMode,
    rate: u32,
) -> Vec<SaveInfo> {
    let (tx, mut rx) = mpsc::unbounded_channel();
    let threads = threads.max(1);
    let mut results = Vec::new();
    let sender = tx.clone();
    let ticker = (rate > 0).then(|| {
        Arc::new(Mutex::new(interval(Duration::from_secs_f64(
            1.0 / rate as f64,
        ))))
    });
    let scan_handle = tokio::spawn(async move {
        futures::stream::iter(urls)
            .map(|url| {
                let client = client.clone();
                let fps = Arc::clone(&fingerprint);
                let status_code = Arc::clone(&status_code);
                let tx = sender.clone();
                let ticker = ticker.clone();

                async move {
                    if let Some(ticker) = ticker {
                        let mut guard = ticker.lock().await;
                        guard.tick().await;
                    }

                    if let Ok(result) = scan_one(&client, &url, &fps, &status_code, mode).await {
                        let _ = tx.send(result);
                    }
                }
            })
            .buffer_unordered(threads)
            .for_each(|_| async {})
            .await
    });

    drop(tx);

    while let Some(save_info) = rx.recv().await {
        println!(
            "{} [{}] [{}] [{}] [{}] [{}]",
            save_info.url,
            save_info.status,
            save_info.title,
            save_info.server,
            save_info.content_length,
            save_info.cms,
        );
        results.push(save_info);
    }

    let _ = scan_handle.await;
    results
}
