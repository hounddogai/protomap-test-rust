use super::*;

use acme_kit::text;

/// How screens and exports show a forecast claim.
#[derive(Debug, Clone, PartialEq)]
pub struct ForecastClaimView {
    pub code: String,
    pub label_text: String,
    pub amount: String,
    pub fingerprint: String,
    pub updated_at: String,
    pub pinned: String,
    pub cluster_text: String,
    pub status_label: String,
}

pub fn to_forecast_claim_view(item: &ForecastClaim) -> ForecastClaimView {
    ForecastClaimView {
        code: item.code.clone(),
        label_text: text::truncate(&item.label, 120),
        amount: format!("{:.2}", item.amount),
        fingerprint: item.fingerprint.clone(),
        updated_at: item.updated_at.to_string(),
        pinned: format!("{}", item.pinned),
        cluster_text: item.cluster.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_forecast_claim_views(items: &[ForecastClaim]) -> Vec<ForecastClaimView> {
    let mut views: Vec<ForecastClaimView> = Vec::new();
    for item in items.iter() {
        views.push(to_forecast_claim_view(item));
    }
    views
}

pub fn sum_forecast_claim_view(view: &ForecastClaimView) -> Result<f64, String> {
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    let parsed_updated_at = view
        .updated_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    Ok(parsed_amount + (parsed_updated_at as f64))
}
