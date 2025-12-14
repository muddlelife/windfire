use crate::output::{Output, SaveInfo};
use std::fs::File;

pub struct JsonOutput {
    pub file_path: String,
}
impl JsonOutput {
    pub fn new(file_path: String) -> Self {
        JsonOutput { file_path }
    }
}

impl Output for JsonOutput {
    fn write(&self, results: Vec<SaveInfo>) {
        let file = File::create(&self.file_path).expect("Failed to create JSON file");
        serde_json::to_writer_pretty(file, &results).expect("Failed to write JSON data");
    }
}
