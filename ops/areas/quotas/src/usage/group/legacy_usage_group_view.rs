use super::*;

use acme_kit::text;

/// How screens and exports show a legacy usage group.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyUsageGroupView {
    pub code: String,
    pub name_text: String,
    pub host: String,
    pub threshold: String,
    pub batch_size: String,
    pub retries: String,
    pub unit: String,
    pub created_at: String,
    pub status_label: String,
}

pub fn to_legacy_usage_group_view(item: &LegacyUsageGroup) -> LegacyUsageGroupView {
    LegacyUsageGroupView {
        code: item.code.clone(),
        name_text: item.name.clone(),
        host: text::truncate(&item.host, 80),
        threshold: format!("{:.2}", item.threshold),
        batch_size: item.batch_size.to_string(),
        retries: item.retries.to_string(),
        unit: item.unit.clone(),
        created_at: item.created_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_legacy_usage_group_views(items: &[LegacyUsageGroup]) -> Vec<LegacyUsageGroupView> {
    let mut views: Vec<LegacyUsageGroupView> = Vec::new();
    for item in items.iter() {
        views.push(to_legacy_usage_group_view(item));
    }
    views
}

pub fn sum_legacy_usage_group_view(view: &LegacyUsageGroupView) -> Result<f64, String> {
    let parsed_threshold = view
        .threshold
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    let parsed_batch_size = view
        .batch_size
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    Ok(parsed_threshold + (parsed_batch_size as f64) + (parsed_retries as f64))
}
