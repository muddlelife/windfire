use crate::fingerprint::model::{Fingerprint, MatchLocation, MatchMethod};
use crate::http::response::ResponseInfo;

impl Fingerprint {
    // 指纹匹配
    pub fn matches(&self, resp: &ResponseInfo) -> bool {
        match self.method {
            MatchMethod::Faviconhash => resp
                .favicon_hash
                .as_ref()
                .is_some_and(|hash| self.keyword.iter().all(|k| k == hash)),

            MatchMethod::Keyword => {
                let target = match self.location {
                    MatchLocation::Header => &resp.headers,
                    MatchLocation::Body => &resp.body,
                };
                self.keyword.iter().all(|k| target.contains(k))
            }
        }
    }
}
