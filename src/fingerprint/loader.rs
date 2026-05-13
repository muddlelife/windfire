use crate::fingerprint::matcher::FingerprintMatcher;
use crate::fingerprint::model::Fingerprint;
use std::error::Error;

pub fn load_fingerprints_from_str(json: &str) -> Result<FingerprintMatcher, Box<dyn Error>> {
    let configs: Vec<Fingerprint> = serde_json::from_str(json)?;
    Ok(FingerprintMatcher::new(configs))
}
