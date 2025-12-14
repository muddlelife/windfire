mod csv;
mod json;

use crate::output::csv::CsvOutput;
use crate::output::json::JsonOutput;
use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct SaveInfo {
    pub url: String,
    pub status: u16,
    pub title: String,
    pub server: String,
    pub jump_url: String, // 跳转后的url
    pub content_length: usize,
    pub cms: String,
}

pub trait Output {
    fn write(&self, results: Vec<SaveInfo>);
}

pub fn output_results(results: Vec<SaveInfo>, output: Option<String>) {
    if let Some(output) = output {
        if output.ends_with("csv") {
            let csv_output = CsvOutput::new(output);
            csv_output.write(results);
        } else {
            let json_output = JsonOutput::new(output);
            json_output.write(results);
        }
    }
}
