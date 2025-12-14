use crate::fingerprint::model::Fingerprint;
use crate::output::SaveInfo;
use crate::scan::http_scan::scan_one;
use futures::StreamExt;
use reqwest::Client;
use std::sync::Arc;
use tokio::sync::mpsc;

pub async fn run(
    client: Client,
    urls: Vec<String>,
    fingerprint: Arc<Vec<Fingerprint>>,
    threads: usize,
    status_code: Arc<Vec<u16>>,
) -> Vec<SaveInfo> {
    let (tx, mut rx) = mpsc::unbounded_channel();
    let threads = threads.max(1);
    let mut results = Vec::new();
    let value = tx.clone();
    let scan_handle = tokio::spawn(async move {
        futures::stream::iter(urls)
            .map(|url| {
                let client = client.clone();
                let fps = Arc::clone(&fingerprint);
                let status_code = Arc::clone(&status_code);
                let tx = value.clone();

                async move {
                    match scan_one(&client, &url, &fps, &status_code).await {
                        Ok(result) => {
                            let _ = tx.send(result.clone());
                            Some(result)
                        }
                        Err(_) => None,
                    }
                }
            })
            .buffer_unordered(threads)
            .collect::<Vec<_>>()
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
