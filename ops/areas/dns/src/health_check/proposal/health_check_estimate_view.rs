use super::*;

use acme_kit::text;

/// How screens and exports show a health check estimate.
#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckEstimateView {
    pub code_text: String,
    pub label: String,
    pub ends_at: String,
    pub unit: String,
    pub name: String,
    pub currency: String,
    pub region_text: String,
    pub key_id: String,
    pub maximum: String,
    pub status_label: String,
}

pub fn to_health_check_estimate_view(item: &HealthCheckEstimate) -> HealthCheckEstimateView {
    HealthCheckEstimateView {
        code_text: item.code.clone(),
        label: item.label.clone(),
        ends_at: item.ends_at.to_string(),
        unit: item.unit.clone(),
        name: item.name.clone(),
        currency: text::truncate(&item.currency, 64),
        region_text: text::truncate(&item.region, 80),
        key_id: item.key_id.clone(),
        maximum: format!("{:.2}", item.maximum),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_health_check_estimate_views(items: &[HealthCheckEstimate]) -> Vec<HealthCheckEstimateView> {
    let mut views: Vec<HealthCheckEstimateView> = Vec::new();
    for item in items.iter() {
        views.push(to_health_check_estimate_view(item));
    }
    views
}

pub fn sum_health_check_estimate_view(view: &HealthCheckEstimateView) -> Result<f64, String> {
    let parsed_ends_at = view
        .ends_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    Ok(parsed_ends_at as f64 + parsed_maximum)
}
