use super::*;

use acme_kit::text;

/// How screens and exports show a strict canary checkpoint.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictCanaryCheckpointView {
    pub code_text: String,
    pub name: String,
    pub retries_text: String,
    pub limit: String,
    pub ceiling_text: String,
    pub duration_minutes_text: String,
    pub score_text: String,
    pub status_label: String,
}

pub fn to_strict_canary_checkpoint_view(item: &StrictCanaryCheckpoint) -> StrictCanaryCheckpointView {
    StrictCanaryCheckpointView {
        code_text: item.code.clone(),
        name: text::truncate(&item.name, 40),
        retries_text: item.retries.to_string(),
        limit: item.limit.to_string(),
        ceiling_text: format!("{:.2}", item.ceiling),
        duration_minutes_text: item.duration_minutes.to_string(),
        score_text: format!("{:.2}", item.score),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_strict_canary_checkpoint_views(items: &[StrictCanaryCheckpoint]) -> Vec<StrictCanaryCheckpointView> {
    let mut views: Vec<StrictCanaryCheckpointView> = Vec::new();
    for item in items.iter() {
        views.push(to_strict_canary_checkpoint_view(item));
    }
    views
}

pub fn sum_strict_canary_checkpoint_view(view: &StrictCanaryCheckpointView) -> Result<f64, String> {
    let parsed_retries = view
        .retries_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    let parsed_ceiling = view
        .ceiling_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    Ok(parsed_retries as f64 + (parsed_limit as f64) + parsed_ceiling)
}
