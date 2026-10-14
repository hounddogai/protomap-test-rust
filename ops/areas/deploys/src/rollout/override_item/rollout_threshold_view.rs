use super::*;

/// How screens and exports show a rollout threshold.
#[derive(Debug, Clone, PartialEq)]
pub struct RolloutThresholdView {
    pub code_text: String,
    pub name: String,
    pub approved: String,
    pub notes: String,
    pub country_text: String,
    pub ratio: String,
    pub batch_size_text: String,
    pub cipher_text: String,
    pub status_label: String,
}

pub fn to_rollout_threshold_view(item: &RolloutThreshold) -> RolloutThresholdView {
    RolloutThresholdView {
        code_text: item.code.clone(),
        name: item.name.clone(),
        approved: format!("{}", item.approved),
        notes: item.notes.clone(),
        country_text: item.country.clone(),
        ratio: format!("{:.2}", item.ratio),
        batch_size_text: item.batch_size.to_string(),
        cipher_text: item.cipher.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rollout_threshold_views(items: &[RolloutThreshold]) -> Vec<RolloutThresholdView> {
    let mut views: Vec<RolloutThresholdView> = Vec::new();
    for item in items.iter() {
        views.push(to_rollout_threshold_view(item));
    }
    views
}

pub fn sum_rollout_threshold_view(view: &RolloutThresholdView) -> Result<f64, String> {
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    let parsed_batch_size = view
        .batch_size_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    Ok(parsed_ratio + (parsed_batch_size as f64))
}
