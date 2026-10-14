use super::*;

use acme_kit::text;

/// How screens and exports show a weighted runbook request.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedRunbookRequestView {
    pub code: String,
    pub label: String,
    pub attempts_text: String,
    pub cipher: String,
    pub threshold: String,
    pub source: String,
    pub status_label: String,
}

pub fn to_weighted_runbook_request_view(item: &WeightedRunbookRequest) -> WeightedRunbookRequestView {
    WeightedRunbookRequestView {
        code: item.code.clone(),
        label: text::truncate(&item.label, 80),
        attempts_text: item.attempts.to_string(),
        cipher: text::truncate(&item.cipher, 120),
        threshold: format!("{:.2}", item.threshold),
        source: text::truncate(&item.source, 80),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_weighted_runbook_request_views(items: &[WeightedRunbookRequest]) -> Vec<WeightedRunbookRequestView> {
    let mut views: Vec<WeightedRunbookRequestView> = Vec::new();
    for item in items.iter() {
        views.push(to_weighted_runbook_request_view(item));
    }
    views
}

pub fn sum_weighted_runbook_request_view(view: &WeightedRunbookRequestView) -> Result<f64, String> {
    let parsed_attempts = view
        .attempts_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    let parsed_threshold = view
        .threshold
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    Ok(parsed_attempts as f64 + parsed_threshold)
}
