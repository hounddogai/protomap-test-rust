use super::*;

use acme_kit::text;

/// How screens and exports show a probe mapping.
#[derive(Debug, Clone, PartialEq)]
pub struct ProbeMappingView {
    pub code: String,
    pub label: String,
    pub tenant_id: String,
    pub retries: String,
    pub namespace: String,
    pub country: String,
    pub quota_units_text: String,
    pub amount: String,
    pub window_days: String,
    pub status_label: String,
}

pub fn to_probe_mapping_view(item: &ProbeMapping) -> ProbeMappingView {
    ProbeMappingView {
        code: item.code.clone(),
        label: item.label.clone(),
        tenant_id: text::truncate(&item.tenant_id, 64),
        retries: item.retries.to_string(),
        namespace: text::truncate(&item.namespace, 40),
        country: item.country.clone(),
        quota_units_text: item.quota_units.to_string(),
        amount: format!("{:.2}", item.amount),
        window_days: item.window_days.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_probe_mapping_views(items: &[ProbeMapping]) -> Vec<ProbeMappingView> {
    let mut views: Vec<ProbeMappingView> = Vec::new();
    for item in items.iter() {
        views.push(to_probe_mapping_view(item));
    }
    views
}

pub fn sum_probe_mapping_view(view: &ProbeMappingView) -> Result<f64, String> {
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    let parsed_quota_units = view
        .quota_units_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    Ok(parsed_retries as f64 + (parsed_quota_units as f64) + parsed_amount)
}
