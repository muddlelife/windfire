use clap::Parser;
use std::sync::Arc;
use windfire::cli::cli_options::Args;
use windfire::fingerprint::loader::load_fingerprints_from_str;
use windfire::http::client::create_http_client;
use windfire::input::url::build_urls;
use windfire::output::output_results;
use windfire::runner::executor::run;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // 生成url列表
    let urls = build_urls(&args)?;
    if urls.is_empty() {
        println!("No URLs found.");
        return Ok(());
    }
    // 加载指纹库
    let finger_json = include_str!("finger.json");
    let fingerprints = load_fingerprints_from_str(finger_json)?;
    let fingerprints = Arc::new(fingerprints);

    // 创建http client
    let client = create_http_client(args.timeout, args.proxy)?;

    let status_code = Arc::new(args.status_code);

    let results = run(client, urls, fingerprints, args.thread, status_code).await;

    output_results(results, args.output);
    Ok(())
}
