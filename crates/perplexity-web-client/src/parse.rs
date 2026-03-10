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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_answer() {
        let json = r#"{"answer": "Hello world"}"#;
        let event = parse_sse_event(json).unwrap();
        assert_eq!(event.answer, Some("Hello world".to_string()));
        assert!(event.web_results.is_empty());
    }

    #[test]
    fn backend_uuid_extraction() {
        let json = r#"{"answer": "Test", "backend_uuid": "abc-123"}"#;
        let event = parse_sse_event(json).unwrap();
        assert_eq!(event.backend_uuid, Some("abc-123".to_string()));
    }

    #[test]
    fn attachments_extraction() {
        let json = r#"{"answer": "Test", "attachments": ["url1", "url2"]}"#;
        let event = parse_sse_event(json).unwrap();
        assert_eq!(event.attachments, vec!["url1", "url2"]);
    }

    #[test]
    fn nested_text_with_final_step() {
        let inner = r#"{"answer": "Nested", "web_results": [{"name": "Src", "url": "https://example.com", "snippet": "Ex"}]}"#;
        let text_content = serde_json::json!([
            {"step_type": "SEARCH", "content": {}},
            {"step_type": "FINAL", "content": {"answer": inner}}
        ]);
        let json = serde_json::json!({"text": serde_json::to_string(&text_content).unwrap()});

        let event = parse_sse_event(&json.to_string()).unwrap();
        assert_eq!(event.answer, Some("Nested".to_string()));
        assert_eq!(event.web_results.len(), 1);
        assert_eq!(event.web_results[0].url, "https://example.com");
    }

    #[test]
    fn fallback_to_top_level_answer() {
        let text_content = serde_json::json!([{"step_type": "SEARCH", "content": {}}]);
        let json = serde_json::json!({
            "text": serde_json::to_string(&text_content).unwrap(),
            "answer": "Top level"
        });
        let event = parse_sse_event(&json.to_string()).unwrap();
        assert_eq!(event.answer, Some("Top level".to_string()));
        assert!(event.web_results.is_empty());
    }

    #[test]
    fn raw_excludes_extracted_keys() {
        let json = r#"{"answer": "A", "backend_uuid": "B", "attachments": [], "extra": 1}"#;
        let event = parse_sse_event(json).unwrap();
        assert!(!event.raw.contains_key("answer"));
        assert!(!event.raw.contains_key("backend_uuid"));
        assert!(event.raw.contains_key("extra"));
    }

    #[test]
    fn empty_event() {
        let event = parse_sse_event("{}").unwrap();
        assert!(event.answer.is_none());
        assert!(event.web_results.is_empty());
    }

    #[test]
    fn invalid_json_errors() {
        assert!(parse_sse_event("not json").is_err());
    }
}
