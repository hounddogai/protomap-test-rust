use super::*;

use acme_kit::text;

/// How screens and exports show a action item exception.
#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemExceptionView {
    pub code_text: String,
    pub title_text: String,
    pub updated_at: String,
    pub retries: String,
    pub sequence: String,
    pub expires_at: String,
    pub segment: String,
    pub checksum: String,
    pub status_label: String,
}

pub fn to_action_item_exception_view(item: &ActionItemException) -> ActionItemExceptionView {
    ActionItemExceptionView {
        code_text: item.code.clone(),
        title_text: item.title.clone(),
        updated_at: item.updated_at.to_string(),
        retries: item.retries.to_string(),
        sequence: item.sequence.to_string(),
        expires_at: item.expires_at.to_string(),
        segment: item.segment.clone(),
        checksum: text::truncate(&item.checksum, 64),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_action_item_exception_views(items: &[ActionItemException]) -> Vec<ActionItemExceptionView> {
    let mut views: Vec<ActionItemExceptionView> = Vec::new();
    for item in items.iter() {
        views.push(to_action_item_exception_view(item));
    }
    views
}

pub fn sum_action_item_exception_view(view: &ActionItemExceptionView) -> Result<f64, String> {
    let parsed_updated_at = view
        .updated_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    let parsed_sequence = view
        .sequence
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    Ok(parsed_updated_at as f64 + (parsed_retries as f64) + (parsed_sequence as f64))
}
