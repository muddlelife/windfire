use crate::fingerprint::model::Fingerprint;
use std::error::Error;

pub fn load_fingerprints_from_str(json: &str) -> Result<Vec<Fingerprint>, Box<dyn Error>> {
    let configs: Vec<Fingerprint> = serde_json::from_str(json)?;

    let mut result = Vec::with_capacity(configs.len());

    for cfg in configs {
        let fp =
            Fingerprint::try_from(cfg).map_err(|e| format!("fingerprint parse error: {}", e))?;
        result.push(fp);
    }

    Ok(result)
}
