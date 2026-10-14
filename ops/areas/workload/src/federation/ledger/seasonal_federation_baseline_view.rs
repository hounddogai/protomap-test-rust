use super::*;

use acme_kit::text;

/// How screens and exports show a seasonal federation baseline.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalFederationBaselineView {
    pub code: String,
    pub label: String,
    pub reviewer_text: String,
    pub quantity: String,
    pub maximum: String,
    pub fingerprint: String,
    pub retries: String,
    pub duration_minutes: String,
    pub capacity: String,
    pub status_label: String,
}

pub fn to_seasonal_federation_baseline_view(item: &SeasonalFederationBaseline) -> SeasonalFederationBaselineView {
    SeasonalFederationBaselineView {
        code: text::truncate(&item.code, 64),
        label: item.label.clone(),
        reviewer_text: item.reviewer.clone(),
        quantity: item.quantity.to_string(),
        maximum: format!("{:.2}", item.maximum),
        fingerprint: item.fingerprint.clone(),
        retries: item.retries.to_string(),
        duration_minutes: item.duration_minutes.to_string(),
        capacity: item.capacity.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_seasonal_federation_baseline_views(
    items: &[SeasonalFederationBaseline],
) -> Vec<SeasonalFederationBaselineView> {
    let mut views: Vec<SeasonalFederationBaselineView> = Vec::new();
    for item in items.iter() {
        views.push(to_seasonal_federation_baseline_view(item));
    }
    views
}

pub fn sum_seasonal_federation_baseline_view(view: &SeasonalFederationBaselineView) -> Result<f64, String> {
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    Ok(parsed_quantity as f64 + parsed_maximum + (parsed_retries as f64))
}
