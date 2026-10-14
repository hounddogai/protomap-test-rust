use super::*;

use acme_kit::text;

/// How screens and exports show a budget burn threshold.
#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnThresholdView {
    pub code: String,
    pub title: String,
    pub locale_text: String,
    pub region: String,
    pub checksum_text: String,
    pub key_id: String,
    pub quota_units_text: String,
    pub active_text: String,
    pub threshold_text: String,
    pub cluster: String,
    pub status_label: String,
}

pub fn to_budget_burn_threshold_view(item: &BudgetBurnThreshold) -> BudgetBurnThresholdView {
    BudgetBurnThresholdView {
        code: item.code.clone(),
        title: text::truncate(&item.title, 80),
        locale_text: text::truncate(&item.locale, 120),
        region: item.region.clone(),
        checksum_text: text::truncate(&item.checksum, 64),
        key_id: item.key_id.clone(),
        quota_units_text: item.quota_units.to_string(),
        active_text: format!("{}", item.active),
        threshold_text: format!("{:.2}", item.threshold),
        cluster: item.cluster.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_budget_burn_threshold_views(items: &[BudgetBurnThreshold]) -> Vec<BudgetBurnThresholdView> {
    let mut views: Vec<BudgetBurnThresholdView> = Vec::new();
    for item in items.iter() {
        views.push(to_budget_burn_threshold_view(item));
    }
    views
}

pub fn sum_budget_burn_threshold_view(view: &BudgetBurnThresholdView) -> Result<f64, String> {
    let parsed_quota_units = view
        .quota_units_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    let parsed_threshold = view
        .threshold_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    Ok(parsed_quota_units as f64 + parsed_threshold)
}
