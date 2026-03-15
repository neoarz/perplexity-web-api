use crate::error::ApiError;
use crate::state::AppState;
use perplexity_web_client::{
    FollowUpContext, ModelPreference, ReasonModel, SearchMode, SearchModel, SearchRequest, Source,
};
use std::sync::Arc;

use super::request::{ApiMode, FollowUpRequest, ImageApiRequest, SearchApiRequest};

pub(crate) struct ResolvedQuery {
    pub search_request: SearchRequest,
    pub api_mode: ApiMode,
    pub mode_str: &'static str,
    pub model_str: String,
}

pub(crate) struct ResolvedImageRequest {
    pub search_request: SearchRequest,
    pub prompt: String,
    pub model_str: String,
}

pub(crate) fn resolve(
    req: SearchApiRequest,
    state: &Arc<AppState>,
) -> Result<ResolvedQuery, ApiError> {
    let query = req.query.trim().to_string();
    if query.is_empty() {
        return Err(ApiError::invalid_request("query can't be empty"));
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

pub(crate) fn resolve_image_request(
    req: ImageApiRequest,
    state: &Arc<AppState>,
) -> Result<ResolvedImageRequest, ApiError> {
    let prompt = req.prompt.trim().to_string();
    if prompt.is_empty() {
        return Err(ApiError::invalid_request("prompt can't be empty"));
    }

    let (mode, preference, model_str) = resolve_search_model(req.model.as_deref(), state)?;

    let search_request = SearchRequest::new(prompt.clone())
        .mode(mode)
        .model(preference)
        .sources(vec![Source::Web])
        .language(req.language)
        .incognito(req.incognito);

    Ok(ResolvedImageRequest {
        search_request,
        prompt,
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
            let (search_mode, pref, model_name) = resolve_search_model(model, state)?;

            Ok((search_mode, Some(pref), "search", model_name))
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
                    "research mode doesn't take a model",
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

fn resolve_search_model(
    model: Option<&str>,
    state: &Arc<AppState>,
) -> Result<(SearchMode, ModelPreference, String), ApiError> {
    let model = match model {
        Some(name) => name
            .parse::<SearchModel>()
            .map_err(ApiError::invalid_model)?,
        None => state.default_search_model.unwrap_or(SearchModel::Sonar),
    };

    let mode = if model == SearchModel::Turbo {
        SearchMode::Auto
    } else {
        SearchMode::Pro
    };

    Ok((mode, model.preference(), model.as_str().to_string()))
}
