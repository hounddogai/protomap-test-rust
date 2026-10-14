use super::*;

use acme_kit::text;

/// How screens and exports show a legacy grant rollup.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyGrantRollupView {
    pub code_text: String,
    pub title_text: String,
    pub maximum: String,
    pub reviewer: String,
    pub audience: String,
    pub fingerprint: String,
    pub burst: String,
    pub unit: String,
    pub key_id: String,
    pub status_label: String,
}

pub fn to_legacy_grant_rollup_view(item: &LegacyGrantRollup) -> LegacyGrantRollupView {
    LegacyGrantRollupView {
        code_text: text::truncate(&item.code, 80),
        title_text: item.title.clone(),
        maximum: format!("{:.2}", item.maximum),
        reviewer: text::truncate(&item.reviewer, 64),
        audience: item.audience.clone(),
        fingerprint: item.fingerprint.clone(),
        burst: item.burst.to_string(),
        unit: text::truncate(&item.unit, 64),
        key_id: text::truncate(&item.key_id, 120),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_legacy_grant_rollup_views(items: &[LegacyGrantRollup]) -> Vec<LegacyGrantRollupView> {
    let mut views: Vec<LegacyGrantRollupView> = Vec::new();
    for item in items.iter() {
        views.push(to_legacy_grant_rollup_view(item));
    }
    views
}

pub fn sum_legacy_grant_rollup_view(view: &LegacyGrantRollupView) -> Result<f64, String> {
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    let parsed_burst = view
        .burst
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    Ok(parsed_maximum + (parsed_burst as f64))
}
