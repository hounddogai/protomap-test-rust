use super::*;

use acme_kit::text;

/// How screens and exports show a pending redaction bundle.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingRedactionBundleView {
    pub code_text: String,
    pub title: String,
    pub score_text: String,
    pub burst_text: String,
    pub floor_value_text: String,
    pub expires_at: String,
    pub cluster: String,
    pub source: String,
    pub rotation_days: String,
    pub status_label: String,
}

pub fn to_pending_redaction_bundle_view(item: &PendingRedactionBundle) -> PendingRedactionBundleView {
    PendingRedactionBundleView {
        code_text: item.code.clone(),
        title: text::truncate(&item.title, 80),
        score_text: format!("{:.2}", item.score),
        burst_text: item.burst.to_string(),
        floor_value_text: format!("{:.2}", item.floor_value),
        expires_at: item.expires_at.to_string(),
        cluster: item.cluster.clone(),
        source: item.source.clone(),
        rotation_days: item.rotation_days.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_pending_redaction_bundle_views(items: &[PendingRedactionBundle]) -> Vec<PendingRedactionBundleView> {
    let mut views: Vec<PendingRedactionBundleView> = Vec::new();
    for item in items.iter() {
        views.push(to_pending_redaction_bundle_view(item));
    }
    views
}

pub fn sum_pending_redaction_bundle_view(view: &PendingRedactionBundleView) -> Result<f64, String> {
    let parsed_score = view
        .score_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    let parsed_burst = view
        .burst_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    let parsed_floor_value = view
        .floor_value_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    Ok(parsed_score + (parsed_burst as f64) + parsed_floor_value)
}
