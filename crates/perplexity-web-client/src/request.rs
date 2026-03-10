use crate::model::{DEEP_RESEARCH_PREFERENCE, ModelPreference, ReasonModel, SearchModel};
use serde::Serialize;
use std::fmt;
use std::str::FromStr;

/// High-level mode for a Perplexity request
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SearchMode {
    /// Uses the default search path
    #[default]
    Auto,
    /// Uses Perplexity's premium search path
    Pro,
    /// Uses a reasoning model
    Reasoning,
    /// Uses deep research
    DeepResearch,
}

impl SearchMode {
    pub(crate) const fn default_preference(&self) -> &'static str {
        match self {
            Self::Auto => SearchModel::Turbo.preference().as_str(),
            Self::Pro => SearchModel::SonarPro.preference().as_str(),
            Self::Reasoning => ReasonModel::SonarReasoning.preference().as_str(),
            Self::DeepResearch => DEEP_RESEARCH_PREFERENCE,
        }
    }
}

/// Source filter passed to Perplexity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Source {
    /// General web results
    #[default]
    Web,
    /// Academic and paper-heavy sources
    Scholar,
    /// Social content
    Social,
}

impl Source {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Web => "web",
            Self::Scholar => "scholar",
            Self::Social => "social",
        }
    }
}

impl fmt::Display for Source {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for Source {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "web" => Ok(Self::Web),
            "scholar" => Ok(Self::Scholar),
            "social" => Ok(Self::Social),
            _ => Err(format!(
                "Unknown source '{s}'. Try one of: web, scholar, social"
            )),
        }
    }
}

/// Values you can feed back into the next request to continue a conversation
#[derive(Debug, Clone, Default, Serialize, serde::Deserialize)]
pub struct FollowUpContext {
    /// Conversation identifier returned by the previous response
    pub backend_uuid: Option<String>,
    /// Attachment URLs that should carry forward into the next turn
    pub attachments: Vec<String>,
}

/// Request builder for one Perplexity query
#[derive(Debug, Clone, Default)]
pub struct SearchRequest {
    /// Natural-language prompt or question
    pub query: String,
    /// Search mode
    pub mode: SearchMode,
    /// Explicit model override
    pub model_preference: Option<ModelPreference>,
    /// Source filters
    pub sources: Vec<Source>,
    /// Request language, usually something like `en-US`
    pub language: String,
    /// Follow-up context from an earlier response
    pub follow_up: Option<FollowUpContext>,
    /// Whether the request should run in incognito mode
    pub incognito: bool,
}

impl SearchRequest {
    pub fn new(query: impl Into<String>) -> Self {
        Self {
            query: query.into(),
            mode: SearchMode::Auto,
            model_preference: None,
            sources: vec![Source::Web],
            language: "en-US".to_string(),
            follow_up: None,
            incognito: true,
        }
    }

    /// Switches the request into a different mode
    pub fn mode(mut self, mode: SearchMode) -> Self {
        self.mode = mode;
        self
    }

    /// Picks a specific model instead of the mode default
    pub fn model(mut self, model: impl Into<ModelPreference>) -> Self {
        self.model_preference = Some(model.into());
        self
    }

    /// Replaces the default source list
    pub fn sources(mut self, sources: Vec<Source>) -> Self {
        self.sources = sources;
        self
    }

    /// Sets the language sent to Perplexity
    pub fn language(mut self, language: impl Into<String>) -> Self {
        self.language = language.into();
        self
    }

    /// Continues a previous conversation using the returned follow-up values
    pub fn follow_up(mut self, context: FollowUpContext) -> Self {
        self.follow_up = Some(context);
        self
    }

    /// Turns incognito mode on or off for this request
    pub fn incognito(mut self, incognito: bool) -> Self {
        self.incognito = incognito;
        self
    }
}

#[derive(Serialize)]
pub(crate) struct AskPayload<'a> {
    pub query_str: &'a str,
    pub params: AskParams<'a>,
}

#[derive(Serialize)]
pub(crate) struct AskParams<'a> {
    pub attachments: Vec<String>,
    pub frontend_context_uuid: String,
    pub frontend_uuid: String,
    pub is_incognito: bool,
    pub language: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_backend_uuid: Option<String>,
    pub mode: &'static str,
    pub model_preference: &'static str,
    pub source: &'static str,
    pub sources: Vec<&'static str>,
    pub version: &'static str,
}
