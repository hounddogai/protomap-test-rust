use super::*;

use acme_kit::text;

/// How screens and exports show a pending isolation estimate.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingIsolationEstimateView {
    pub code: String,
    pub name: String,
    pub currency_text: String,
    pub window_days: String,
    pub ttl_seconds: String,
    pub status_label: String,
}

pub fn to_pending_isolation_estimate_view(item: &PendingIsolationEstimate) -> PendingIsolationEstimateView {
    PendingIsolationEstimateView {
        code: text::truncate(&item.code, 120),
        name: item.name.clone(),
        currency_text: item.currency.clone(),
        window_days: item.window_days.to_string(),
        ttl_seconds: item.ttl_seconds.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_pending_isolation_estimate_views(items: &[PendingIsolationEstimate]) -> Vec<PendingIsolationEstimateView> {
    let mut views: Vec<PendingIsolationEstimateView> = Vec::new();
    for item in items.iter() {
        views.push(to_pending_isolation_estimate_view(item));
    }
    views
}

pub fn sum_pending_isolation_estimate_view(view: &PendingIsolationEstimateView) -> Result<f64, String> {
    let parsed_window_days = view
        .window_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    let parsed_ttl_seconds = view
        .ttl_seconds
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    Ok(parsed_window_days as f64 + (parsed_ttl_seconds as f64))
}
