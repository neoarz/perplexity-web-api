use crate::error::ApiError;
use crate::state::AppState;
use perplexity_web_client::{
    FollowUpContext, ModelPreference, ReasonModel, SearchMode, SearchModel, SearchRequest, Source,
};
use std::collections::HashSet;
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
    let SearchApiRequest {
        query,
        mode: api_mode,
        model,
        sources,
        language,
        incognito,
        attachments,
        follow_up,
    } = req;

    let query = query.trim().to_string();
    if query.is_empty() {
        return Err(ApiError::invalid_request("query can't be empty"));
    }

    let sources = parse_sources(&sources)?;
    let attachments = dedupe_attachments(attachments);
    let follow_up = merge_follow_up_attachments(follow_up, attachments);
    if api_mode != ApiMode::Search
        && follow_up
            .as_ref()
            .is_some_and(|ctx| !ctx.attachments.is_empty())
    {
        return Err(ApiError::invalid_request(
            "attachments are only supported in search mode",
        ));
    }

    let (mode, preference, mode_str, model_str) =
        resolve_mode_and_model(api_mode, model.as_deref(), state)?;

    let mut search_request = SearchRequest::new(query)
        .mode(mode)
        .sources(sources)
        .language(language)
        .incognito(incognito);

    if let Some(pref) = preference {
        search_request = search_request.model(pref);
    }

    if let Some(ctx) = follow_up {
        search_request = search_request.follow_up(ctx);
    }

    Ok(ResolvedQuery {
        search_request,
        api_mode,
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

fn merge_follow_up_attachments(
    follow_up: Option<FollowUpRequest>,
    attachments: Vec<String>,
) -> Option<FollowUpContext> {
    let Some(follow_up) = follow_up else {
        return (!attachments.is_empty()).then_some(FollowUpContext {
            backend_uuid: None,
            attachments,
        });
    };

    let merged = dedupe_attachments(
        follow_up
            .attachments
            .into_iter()
            .chain(attachments)
            .collect(),
    );

    Some(FollowUpContext {
        backend_uuid: follow_up.backend_uuid,
        attachments: merged,
    })
}

fn dedupe_attachments(attachments: Vec<String>) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut deduped = Vec::new();

    for attachment in attachments {
        let attachment = attachment.trim();
        if attachment.is_empty() {
            continue;
        }

        if seen.insert(attachment.to_string()) {
            deduped.push(attachment.to_string());
        }
    }

    deduped
}
