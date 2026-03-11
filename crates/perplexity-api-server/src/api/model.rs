use crate::error::ApiError;
use crate::state::AppState;
use perplexity_web_client::{
    FollowUpContext, ModelPreference, ReasonModel, SearchMode, SearchModel, SearchRequest, Source,
};
use std::sync::Arc;

use super::request::{ApiMode, FollowUpRequest, SearchApiRequest};

pub struct ResolvedQuery {
    pub search_request: SearchRequest,
    pub api_mode: ApiMode,
    pub mode_str: &'static str,
    pub model_str: String,
}

pub fn resolve(req: SearchApiRequest, state: &Arc<AppState>) -> Result<ResolvedQuery, ApiError> {
    let query = req.query.trim().to_string();
    if query.is_empty() {
        return Err(ApiError::invalid_request("Query can't be empty"));
    }

    let sources = parse_sources(&req.sources)?;
    let (mode, preference, mode_str, model_str) =
        resolve_mode_and_model(req.mode, req.model.as_deref(), state)?;

    let follow_up = req.follow_up.map(follow_up_from_request);

    let mut search_request = SearchRequest::new(query)
        .mode(mode)
        .sources(sources)
        .language(req.language)
        .incognito(req.incognito);

    if let Some(pref) = preference {
        search_request = search_request.model(pref);
    }

    if let Some(ctx) = follow_up {
        search_request = search_request.follow_up(ctx);
    }

    Ok(ResolvedQuery {
        search_request,
        api_mode: req.mode,
        mode_str,
        model_str,
    })
}

fn parse_sources(raw: &[String]) -> Result<Vec<Source>, ApiError> {
    if raw.is_empty() {
        return Ok(vec![Source::Web]);
    }

    raw.iter()
        .map(|s| s.parse::<Source>().map_err(ApiError::invalid_request))
        .collect()
}

fn resolve_mode_and_model(
    mode: ApiMode,
    model: Option<&str>,
    state: &Arc<AppState>,
) -> Result<(SearchMode, Option<ModelPreference>, &'static str, String), ApiError> {
    match mode {
        ApiMode::Search => {
            let (pref, model_name) = match model {
                Some(name) => {
                    let m: SearchModel = name
                        .parse()
                        .map_err(|e: String| ApiError::invalid_model(e))?;
                    (Some(m.preference()), m.as_str().to_string())
                }
                None => match state.default_search_model {
                    Some(m) => (Some(m.preference()), m.as_str().to_string()),
                    None => (
                        Some(SearchModel::Sonar.preference()),
                        SearchModel::Sonar.as_str().to_string(),
                    ),
                },
            };

            let search_mode = if pref.is_some() && pref.map(|p| p.as_str()) != Some("turbo") {
                SearchMode::Pro
            } else {
                SearchMode::Auto
            };

            Ok((search_mode, pref, "search", model_name))
        }
        ApiMode::Reason => {
            let (pref, model_name) = match model {
                Some(name) => {
                    let m: ReasonModel = name
                        .parse()
                        .map_err(|e: String| ApiError::invalid_model(e))?;
                    (Some(m.preference()), m.as_str().to_string())
                }
                None => match state.default_reason_model {
                    Some(m) => (Some(m.preference()), m.as_str().to_string()),
                    None => (None, "sonar-reasoning".to_string()),
                },
            };

            Ok((SearchMode::Reasoning, pref, "reason", model_name))
        }
        ApiMode::Research => {
            if model.is_some() {
                return Err(ApiError::invalid_request(
                    "Research mode doesn't take a model",
                ));
            }
            Ok((
                SearchMode::DeepResearch,
                None,
                "research",
                "pplx_alpha".to_string(),
            ))
        }
    }
}

fn follow_up_from_request(req: FollowUpRequest) -> FollowUpContext {
    FollowUpContext {
        backend_uuid: req.backend_uuid,
        attachments: req.attachments,
    }
}
