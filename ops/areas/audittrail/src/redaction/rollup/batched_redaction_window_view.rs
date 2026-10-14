use super::*;

use acme_kit::text;

/// How screens and exports show a batched redaction window.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedRedactionWindowView {
    pub code: String,
    pub name_text: String,
    pub audience_text: String,
    pub cluster_text: String,
    pub duration_minutes: String,
    pub label: String,
    pub updated_at: String,
    pub source: String,
    pub sequence_text: String,
    pub visible_text: String,
    pub status_label: String,
}

pub fn to_batched_redaction_window_view(item: &BatchedRedactionWindow) -> BatchedRedactionWindowView {
    BatchedRedactionWindowView {
        code: item.code.clone(),
        name_text: text::truncate(&item.name, 80),
        audience_text: text::truncate(&item.audience, 40),
        cluster_text: item.cluster.clone(),
        duration_minutes: item.duration_minutes.to_string(),
        label: item.label.clone(),
        updated_at: item.updated_at.to_string(),
        source: item.source.clone(),
        sequence_text: item.sequence.to_string(),
        visible_text: format!("{}", item.visible),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_batched_redaction_window_views(items: &[BatchedRedactionWindow]) -> Vec<BatchedRedactionWindowView> {
    let mut views: Vec<BatchedRedactionWindowView> = Vec::new();
    for item in items.iter() {
        views.push(to_batched_redaction_window_view(item));
    }
    views
}

pub fn sum_batched_redaction_window_view(view: &BatchedRedactionWindowView) -> Result<f64, String> {
    let parsed_duration_minutes = view
        .duration_minutes
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    let parsed_updated_at = view
        .updated_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    let parsed_sequence = view
        .sequence_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    Ok(parsed_duration_minutes as f64 + (parsed_updated_at as f64) + (parsed_sequence as f64))
}
