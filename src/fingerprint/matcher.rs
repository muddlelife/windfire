use crate::fingerprint::model::{Fingerprint, MatchLocation, MatchMethod};
use crate::http::response::ResponseInfo;

#[derive(Debug, Clone)]
struct CompiledFingerprint {
    cms: String,
    keywords: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct FingerprintMatcher {
    favicon_rules: Vec<CompiledFingerprint>,
    header_rules: Vec<CompiledFingerprint>,
    body_rules: Vec<CompiledFingerprint>,
}

impl FingerprintMatcher {
    pub fn new(fingerprints: Vec<Fingerprint>) -> Self {
        let mut favicon_rules = Vec::new();
        let mut header_rules = Vec::new();
        let mut body_rules = Vec::new();

        for fp in fingerprints {
            let compiled = CompiledFingerprint {
                cms: fp.cms,
                keywords: fp.keyword,
            };

            match (fp.method, fp.location) {
                (MatchMethod::Faviconhash, _) => favicon_rules.push(compiled),
                (MatchMethod::Keyword, MatchLocation::Header) => header_rules.push(compiled),
                (MatchMethod::Keyword, MatchLocation::Body) => body_rules.push(compiled),
            }
        }

        Self {
            favicon_rules,
            header_rules,
            body_rules,
        }
    }

    pub fn match_response(&self, resp: &ResponseInfo) -> Vec<String> {
        let mut matched = Vec::new();

        if let Some(hash) = resp.favicon_hash.as_deref() {
            for rule in &self.favicon_rules {
                if rule.keywords.iter().all(|k| k == hash) {
                    matched.push(rule.cms.clone());
                }
            }
        }

        self.match_keyword_rules(&self.header_rules, &resp.headers, &mut matched);
        self.match_keyword_rules(&self.body_rules, &resp.body, &mut matched);

        matched.sort();
        matched.dedup();
        matched
    }

    fn match_keyword_rules(
        &self,
        rules: &[CompiledFingerprint],
        target: &str,
        matched: &mut Vec<String>,
    ) {
        for rule in rules {
            if rule.keywords.iter().all(|k| target.contains(k)) {
                matched.push(rule.cms.clone());
            }
        }
    }
}
