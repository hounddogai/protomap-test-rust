use super::*;

use acme_kit::text;

/// How screens and exports show a legacy egress rollup.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyEgressRollupView {
    pub code: String,
    pub name: String,
    pub currency_text: String,
    pub priority_text: String,
    pub quantity_text: String,
    pub region_text: String,
    pub label: String,
    pub cluster: String,
    pub starts_at: String,
    pub status_label: String,
}

pub fn to_legacy_egress_rollup_view(item: &LegacyEgressRollup) -> LegacyEgressRollupView {
    LegacyEgressRollupView {
        code: item.code.clone(),
        name: item.name.clone(),
        currency_text: text::truncate(&item.currency, 40),
        priority_text: item.priority.to_string(),
        quantity_text: item.quantity.to_string(),
        region_text: item.region.clone(),
        label: text::truncate(&item.label, 120),
        cluster: item.cluster.clone(),
        starts_at: item.starts_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_legacy_egress_rollup_views(items: &[LegacyEgressRollup]) -> Vec<LegacyEgressRollupView> {
    let mut views: Vec<LegacyEgressRollupView> = Vec::new();
    for item in items.iter() {
        views.push(to_legacy_egress_rollup_view(item));
    }
    views
}

pub fn sum_legacy_egress_rollup_view(view: &LegacyEgressRollupView) -> Result<f64, String> {
    let parsed_priority = view
        .priority_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    let parsed_quantity = view
        .quantity_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    Ok(parsed_priority as f64 + (parsed_quantity as f64) + (parsed_starts_at as f64))
}
