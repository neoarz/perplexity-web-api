use crate::api::response::{ModelDefaults, ModelInfo, ModelsApiResponse, ResearchModelInfo};
use crate::http::response::{JsonOutputQuery, json_response};
use crate::state::AppState;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::Response;
use perplexity_web_client::{ReasonModel, SearchModel};
use std::sync::Arc;

pub async fn list_models(
    State(state): State<Arc<AppState>>,
    Query(output): Query<JsonOutputQuery>,
) -> Response {
    let search_models: Vec<ModelInfo> = SearchModel::ALL
        .iter()
        .map(|m| ModelInfo {
            name: m.as_str(),
            description: model_description_search(m),
        })
        .collect();

    let reason_models: Vec<ModelInfo> = ReasonModel::ALL
        .iter()
        .map(|m| ModelInfo {
            name: m.as_str(),
            description: model_description_reason(m),
        })
        .collect();

    let default_search = state
        .default_search_model
        .map(|m| m.as_str().to_string())
        .unwrap_or_else(|| "sonar".to_string());

    let default_reason = state
        .default_reason_model
        .map(|m| m.as_str().to_string())
        .unwrap_or_else(|| "sonar-reasoning".to_string());

    json_response(
        StatusCode::OK,
        &ModelsApiResponse {
            search: search_models,
            reason: reason_models,
            research: ResearchModelInfo {
                name: "pplx_alpha",
                description: "Fixed deep research mode",
            },
            defaults: ModelDefaults {
                search: default_search,
                reason: default_reason,
            },
        },
        output.pretty_enabled(),
    )
}

fn model_description_search(m: &SearchModel) -> &'static str {
    match m {
        SearchModel::Turbo => "Default free model",
        SearchModel::Sonar => "Sonar model",
        SearchModel::SonarPro => "Sonar pro model",
        SearchModel::Gemini30Flash => "Gemini 3.0 Flash",
        SearchModel::Gpt54 => "GPT-5.4",
        SearchModel::Gpt52 => "GPT-5.2",
        SearchModel::Claude46Sonnet => "Claude 4.6 Sonnet",
        SearchModel::Grok41 => "Grok 4.1",
    }
}

fn model_description_reason(m: &ReasonModel) -> &'static str {
    match m {
        ReasonModel::SonarReasoning => "Default reasoning model",
        ReasonModel::Gemini30FlashThinking => "Gemini 3.0 Flash with thinking",
        ReasonModel::Gemini31Pro => "Gemini 3.1 Pro",
        ReasonModel::Gpt54Thinking => "GPT-5.4 with thinking",
        ReasonModel::Gpt52Thinking => "GPT-5.2 with thinking",
        ReasonModel::Claude46SonnetThinking => "Claude 4.6 Sonnet with thinking",
        ReasonModel::Grok41Reasoning => "Grok 4.1 with reasoning",
        ReasonModel::KimiK25Thinking => "Kimi K2.5 with thinking",
    }
}
