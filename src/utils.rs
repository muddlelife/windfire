use tokio::fs::File;
use tokio::io::{AsyncBufReadExt, BufReader};


pub(crate) async fn read_file(path: &str) -> Result<Vec<String>, tokio::io::Error> {
    let file = File::open(path).await.unwrap();
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
        println!("{:?}",urls)
    }
}