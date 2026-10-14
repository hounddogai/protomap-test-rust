use super::*;

use acme_kit::text;

/// How screens and exports show a rolling limit override.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingLimitOverrideView {
    pub code: String,
    pub title_text: String,
    pub region: String,
    pub grace_days: String,
    pub principal: String,
    pub threshold: String,
    pub key_id: String,
    pub namespace_text: String,
    pub status_label: String,
}

pub fn to_rolling_limit_override_view(item: &RollingLimitOverride) -> RollingLimitOverrideView {
    RollingLimitOverrideView {
        code: item.code.clone(),
        title_text: item.title.clone(),
        region: text::truncate(&item.region, 64),
        grace_days: item.grace_days.to_string(),
        principal: text::truncate(&item.principal, 40),
        threshold: format!("{:.2}", item.threshold),
        key_id: text::truncate(&item.key_id, 64),
        namespace_text: item.namespace.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rolling_limit_override_views(items: &[RollingLimitOverride]) -> Vec<RollingLimitOverrideView> {
    let mut views: Vec<RollingLimitOverrideView> = Vec::new();
    for item in items.iter() {
        views.push(to_rolling_limit_override_view(item));
    }
    views
}

pub fn sum_rolling_limit_override_view(view: &RollingLimitOverrideView) -> Result<f64, String> {
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    let parsed_threshold = view
        .threshold
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    Ok(parsed_grace_days as f64 + parsed_threshold)
}
