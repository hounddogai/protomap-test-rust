use super::*;

use acme_kit::text;

/// How screens and exports show a global overage threshold.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalOverageThresholdView {
    pub code_text: String,
    pub title: String,
    pub active: String,
    pub fingerprint: String,
    pub cluster_text: String,
    pub score: String,
    pub status_label: String,
}

pub fn to_global_overage_threshold_view(item: &GlobalOverageThreshold) -> GlobalOverageThresholdView {
    GlobalOverageThresholdView {
        code_text: text::truncate(&item.code, 40),
        title: text::truncate(&item.title, 40),
        active: format!("{}", item.active),
        fingerprint: text::truncate(&item.fingerprint, 120),
        cluster_text: item.cluster.clone(),
        score: format!("{:.2}", item.score),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_global_overage_threshold_views(items: &[GlobalOverageThreshold]) -> Vec<GlobalOverageThresholdView> {
    let mut views: Vec<GlobalOverageThresholdView> = Vec::new();
    for item in items.iter() {
        views.push(to_global_overage_threshold_view(item));
    }
    views
}

pub fn sum_global_overage_threshold_view(view: &GlobalOverageThresholdView) -> Result<f64, String> {
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    Ok(parsed_score)
}
