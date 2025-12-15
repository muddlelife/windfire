use crate::output::{Output, SaveInfo};
use csv::WriterBuilder;
use std::fs::File;
use std::io::Write;

pub struct CsvOutput {
    pub file_path: String,
}
impl CsvOutput {
    pub fn new(file_path: String) -> CsvOutput {
        CsvOutput { file_path }
    }
}
impl Output for CsvOutput {
    fn write(&self, results: Vec<SaveInfo>) {
        let mut file = File::create(&self.file_path).expect("Failed to create CSV file");
        file.write_all(b"\xEF\xBB\xBF")
            .expect("Failed to write BOM");

        let mut wtr = WriterBuilder::new().from_writer(file);

        for record in results {
            if let Err(e) = wtr.serialize(record) {
                panic!("Error writing record: {}", e);
            }
        }

        wtr.flush().expect("Failed to flush CSV writer");
    }
}
