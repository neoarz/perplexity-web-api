use crate::error::{Error, Result};
use crate::response::{GeneratedImage, SearchEvent, SearchWebResult};
use serde_json::{Map, Value};
use std::collections::HashMap;

const EXTRACTED_KEYS: &[&str] = &["answer", "backend_uuid", "attachments"];

pub(crate) fn parse_sse_event(json_str: &str) -> Result<SearchEvent> {
    let mut content: Map<String, Value> = serde_json::from_str(json_str).map_err(Error::Json)?;

    parse_nested_text_field(&mut content);

    let (answer, web_results) = extract_answer_and_web_results(&content);
    let backend_uuid = extract_string(&content, "backend_uuid");
    let attachments = extract_string_array(&content, "attachments");
    let generated_images = extract_generated_images(&content);
    let image_generation = extract_image_generation(&content) || !generated_images.is_empty();
    let raw = build_raw_map(content);

    Ok(SearchEvent {
        answer,
        web_results,
        backend_uuid,
        attachments,
        image_generation,
        generated_images,
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

fn extract_image_generation(content: &Map<String, Value>) -> bool {
    content
        .get("classifier_results")
        .and_then(Value::as_object)
        .and_then(|classifier| classifier.get("image_generation"))
        .and_then(Value::as_bool)
        .unwrap_or(false)
}

fn extract_generated_images(content: &Map<String, Value>) -> Vec<GeneratedImage> {
    let mut images = Vec::new();

    if let Some(media_items) = content.get("media_items").and_then(Value::as_array) {
        for item in media_items {
            if let Some(image) = extract_image_from_media_item(item) {
                merge_generated_image(&mut images, image);
            }
        }
    }

    if let Some(blocks) = content.get("blocks").and_then(Value::as_array) {
        for block in blocks {
            if let Some(assets) = block.get("assets").and_then(Value::as_array) {
                for asset in assets {
                    if let Some(image) = extract_image_from_asset(asset) {
                        merge_generated_image(&mut images, image);
                    }
                }
            }

            if let Some(image_results) = block
                .get("content")
                .and_then(Value::as_object)
                .and_then(|content| content.get("image_results"))
                .and_then(Value::as_array)
            {
                for image_result in image_results {
                    if let Some(image) = extract_image_from_image_result(image_result) {
                        merge_generated_image(&mut images, image);
                    }
                }
            }
        }
    }

    images
}

fn extract_image_from_media_item(item: &Value) -> Option<GeneratedImage> {
    let object = item.as_object()?;
    let url = object.get("image").and_then(Value::as_str)?.to_string();

    Some(GeneratedImage {
        url,
        thumbnail_url: extract_string_from_object(object, "thumbnail"),
        download_url: None,
        mime_type: extract_any_string(object, &["mime_type", "mimetype", "content_type", "type"]),
        source: extract_string_from_object(object, "source"),
        generation_model: object
            .get("generated_media_metadata")
            .and_then(Value::as_object)
            .and_then(|metadata| extract_any_string(metadata, &["model_str", "model"])),
        prompt: extract_any_string(object, &["prompt", "description", "caption"]),
    })
}

fn extract_image_from_asset(asset: &Value) -> Option<GeneratedImage> {
    let object = asset.as_object()?;
    let generated_image = object.get("generated_image").and_then(Value::as_object)?;
    let url = extract_string_from_object(generated_image, "url")?;

    Some(GeneratedImage {
        url,
        thumbnail_url: extract_string_from_object(generated_image, "thumbnail_url"),
        download_url: object
            .get("download_info")
            .and_then(Value::as_array)
            .and_then(|items| items.iter().find_map(extract_download_url)),
        mime_type: extract_any_string(
            generated_image,
            &["mime_type", "mimetype", "content_type", "type"],
        )
        .or_else(|| extract_any_string(object, &["mime_type", "mimetype", "content_type"])),
        source: extract_string_from_object(object, "source"),
        generation_model: object
            .get("generated_media_metadata")
            .and_then(Value::as_object)
            .and_then(|metadata| extract_any_string(metadata, &["model_str", "model"])),
        prompt: extract_any_string(generated_image, &["prompt", "description", "caption"])
            .or_else(|| extract_any_string(object, &["prompt", "description", "caption"])),
    })
}

fn extract_image_from_image_result(image_result: &Value) -> Option<GeneratedImage> {
    let object = image_result.as_object()?;
    let url = extract_string_from_object(object, "url")?;

    Some(GeneratedImage {
        url,
        thumbnail_url: extract_any_string(object, &["thumbnail_url", "thumbnail"]),
        download_url: None,
        mime_type: extract_any_string(object, &["mime_type", "mimetype", "content_type", "type"]),
        source: extract_string_from_object(object, "source"),
        generation_model: object
            .get("generated_media_metadata")
            .and_then(Value::as_object)
            .and_then(|metadata| extract_any_string(metadata, &["model_str", "model"])),
        prompt: extract_any_string(object, &["prompt", "description", "caption"]),
    })
}

fn extract_download_url(value: &Value) -> Option<String> {
    value
        .as_object()
        .and_then(|object| extract_string_from_object(object, "url"))
}

fn merge_generated_image(images: &mut Vec<GeneratedImage>, incoming: GeneratedImage) {
    if let Some(existing) = images.iter_mut().find(|image| image.url == incoming.url) {
        merge_optional_field(&mut existing.thumbnail_url, incoming.thumbnail_url);
        merge_optional_field(&mut existing.download_url, incoming.download_url);
        merge_optional_field(&mut existing.mime_type, incoming.mime_type);
        merge_optional_field(&mut existing.source, incoming.source);
        merge_optional_field(&mut existing.generation_model, incoming.generation_model);
        merge_optional_field(&mut existing.prompt, incoming.prompt);
    } else {
        images.push(incoming);
    }
}

fn merge_optional_field(slot: &mut Option<String>, incoming: Option<String>) {
    if slot.is_none() {
        *slot = incoming;
    }
}

fn extract_string_from_object(
    object: &serde_json::Map<String, Value>,
    key: &str,
) -> Option<String> {
    object
        .get(key)
        .and_then(Value::as_str)
        .map(ToString::to_string)
}

fn extract_any_string(object: &serde_json::Map<String, Value>, keys: &[&str]) -> Option<String> {
    keys.iter()
        .find_map(|key| extract_string_from_object(object, key))
}

fn build_raw_map(content: Map<String, Value>) -> HashMap<String, Value> {
    content
        .into_iter()
        .filter(|(k, _)| !EXTRACTED_KEYS.contains(&k.as_str()))
        .collect()
}
