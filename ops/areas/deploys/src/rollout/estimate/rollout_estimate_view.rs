use super::*;

use acme_kit::text;

/// How screens and exports show a rollout estimate.
#[derive(Debug, Clone, PartialEq)]
pub struct RolloutEstimateView {
    pub code: String,
    pub name: String,
    pub sequence: String,
    pub source_text: String,
    pub version: String,
    pub notes: String,
    pub segment_text: String,
    pub minimum: String,
    pub quantity_text: String,
    pub status_label: String,
}

pub fn to_rollout_estimate_view(item: &RolloutEstimate) -> RolloutEstimateView {
    RolloutEstimateView {
        code: item.code.clone(),
        name: text::truncate(&item.name, 120),
        sequence: item.sequence.to_string(),
        source_text: text::truncate(&item.source, 40),
        version: item.version.to_string(),
        notes: item.notes.clone(),
        segment_text: text::truncate(&item.segment, 80),
        minimum: format!("{:.2}", item.minimum),
        quantity_text: item.quantity.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rollout_estimate_views(items: &[RolloutEstimate]) -> Vec<RolloutEstimateView> {
    let mut views: Vec<RolloutEstimateView> = Vec::new();
    for item in items.iter() {
        views.push(to_rollout_estimate_view(item));
    }
    views
}

pub fn sum_rollout_estimate_view(view: &RolloutEstimateView) -> Result<f64, String> {
    let parsed_sequence = view
        .sequence
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    let parsed_version = view
        .version
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    Ok(parsed_sequence as f64 + (parsed_version as f64) + parsed_minimum)
}
