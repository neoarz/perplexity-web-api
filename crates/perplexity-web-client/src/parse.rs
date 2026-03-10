use crate::error::{Error, Result};
use crate::response::{SearchEvent, SearchWebResult};
use serde_json::{Map, Value};
use std::collections::HashMap;

const EXTRACTED_KEYS: &[&str] = &["answer", "backend_uuid", "attachments"];

pub(crate) fn parse_sse_event(json_str: &str) -> Result<SearchEvent> {
    let mut content: Map<String, Value> = serde_json::from_str(json_str).map_err(Error::Json)?;

    parse_nested_text_field(&mut content);

    let (answer, web_results) = extract_answer_and_web_results(&content);
    let backend_uuid = extract_string(&content, "backend_uuid");
    let attachments = extract_string_array(&content, "attachments");
    let raw = build_raw_map(content);

    Ok(SearchEvent {
        answer,
        web_results,
        backend_uuid,
        attachments,
        raw,
    })
}

fn parse_nested_text_field(content: &mut Map<String, Value>) {
    let Some(text_value) = content.get("text") else {
        return;
    };
    let Some(text_str) = text_value.as_str() else {
        return;
    };
    if let Ok(parsed) = serde_json::from_str::<Value>(text_str) {
        content.insert("text".to_string(), parsed);
    }
}

fn extract_answer_and_web_results(
    content: &Map<String, Value>,
) -> (Option<String>, Vec<SearchWebResult>) {
    if let Some((answer, web_results)) = extract_from_final_step(content) {
        return (answer, web_results);
    }
    let answer = extract_string(content, "answer");
    (answer, Vec::new())
}

fn extract_from_final_step(
    content: &Map<String, Value>,
) -> Option<(Option<String>, Vec<SearchWebResult>)> {
    let text = content.get("text")?;
    let steps = text.as_array()?;

    let final_step = steps
        .iter()
        .find(|step| step.get("step_type").and_then(|v| v.as_str()) == Some("FINAL"))?;

    let step_content = final_step.get("content")?;
    let answer_str = step_content.get("answer")?.as_str()?;
    let answer_data: Value = serde_json::from_str(answer_str).ok()?;

    let answer = answer_data
        .get("answer")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let web_results = answer_data
        .get("web_results")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|v| extract_web_result(&v))
        .collect();

    Some((answer, web_results))
}

fn extract_web_result(value: &Value) -> Option<SearchWebResult> {
    let name = value.get("name").and_then(|v| v.as_str())?.to_string();
    let url = value.get("url").and_then(|v| v.as_str())?.to_string();
    let snippet = value.get("snippet").and_then(|v| v.as_str())?.to_string();
    Some(SearchWebResult { name, url, snippet })
}

fn extract_string(content: &Map<String, Value>, key: &str) -> Option<String> {
    content
        .get(key)
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

fn extract_string_array(content: &Map<String, Value>, key: &str) -> Vec<String> {
    content
        .get(key)
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default()
}

fn build_raw_map(content: Map<String, Value>) -> HashMap<String, Value> {
    content
        .into_iter()
        .filter(|(k, _)| !EXTRACTED_KEYS.contains(&k.as_str()))
        .collect()
}
