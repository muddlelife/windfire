use serde::Deserialize;

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MatchMethod {
    Keyword,
    Faviconhash,
}
#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MatchLocation {
    Header,
    Body,
}

#[derive(Debug, Deserialize)]
pub struct Fingerprint {
    pub cms: String,
    pub method: MatchMethod,
    pub location: MatchLocation,
    pub keyword: Vec<String>,
}
