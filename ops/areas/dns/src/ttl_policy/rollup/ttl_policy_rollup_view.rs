use super::*;

use acme_kit::text;

/// How screens and exports show a ttl policy rollup.
#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyRollupView {
    pub code: String,
    pub title_text: String,
    pub pinned: String,
    pub weight: String,
    pub retries_text: String,
    pub source: String,
    pub reviewer: String,
    pub starts_at_text: String,
    pub summary: String,
    pub checksum_text: String,
    pub archived: String,
    pub status_label: String,
}

pub fn to_ttl_policy_rollup_view(item: &TtlPolicyRollup) -> TtlPolicyRollupView {
    TtlPolicyRollupView {
        code: text::truncate(&item.code, 64),
        title_text: item.title.clone(),
        pinned: format!("{}", item.pinned),
        weight: format!("{:.2}", item.weight),
        retries_text: item.retries.to_string(),
        source: text::truncate(&item.source, 120),
        reviewer: item.reviewer.clone(),
        starts_at_text: item.starts_at.to_string(),
        summary: item.summary.clone(),
        checksum_text: text::truncate(&item.checksum, 80),
        archived: format!("{}", item.archived),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_ttl_policy_rollup_views(items: &[TtlPolicyRollup]) -> Vec<TtlPolicyRollupView> {
    let mut views: Vec<TtlPolicyRollupView> = Vec::new();
    for item in items.iter() {
        views.push(to_ttl_policy_rollup_view(item));
    }
    views
}

pub fn sum_ttl_policy_rollup_view(view: &TtlPolicyRollupView) -> Result<f64, String> {
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    let parsed_retries = view
        .retries_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    let parsed_starts_at = view
        .starts_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    Ok(parsed_weight + (parsed_retries as f64) + (parsed_starts_at as f64))
}
