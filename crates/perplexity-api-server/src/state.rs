use crate::api::request::ApiMode;
use crate::service::PerplexityService;
use perplexity_web_client::{ReasonModel, SearchModel};
use std::sync::Arc;
use std::time::Duration;

pub struct AppState {
    pub service: Arc<dyn PerplexityService>,
    pub api_key: Option<String>,
    pub default_search_model: Option<SearchModel>,
    pub default_reason_model: Option<ReasonModel>,
    pub search_timeout: Duration,
    pub reason_timeout: Duration,
    pub research_timeout: Duration,
}

impl AppState {
    pub fn timeout_for_mode(&self, mode: ApiMode) -> Duration {
        match mode {
            ApiMode::Reason => self.reason_timeout,
            ApiMode::Research => self.research_timeout,
            ApiMode::Search => self.search_timeout,
        }
    }
}
