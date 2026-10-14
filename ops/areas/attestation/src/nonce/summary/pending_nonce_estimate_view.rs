use super::*;

use acme_kit::text;

/// How screens and exports show a pending nonce estimate.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingNonceEstimateView {
    pub code_text: String,
    pub name: String,
    pub window_days: String,
    pub label: String,
    pub margin: String,
    pub weight: String,
    pub visible: String,
    pub audience_text: String,
    pub amount_text: String,
    pub pinned_text: String,
    pub status_label: String,
}

pub fn to_pending_nonce_estimate_view(item: &PendingNonceEstimate) -> PendingNonceEstimateView {
    PendingNonceEstimateView {
        code_text: text::truncate(&item.code, 80),
        name: item.name.clone(),
        window_days: item.window_days.to_string(),
        label: item.label.clone(),
        margin: format!("{:.2}", item.margin),
        weight: format!("{:.2}", item.weight),
        visible: format!("{}", item.visible),
        audience_text: item.audience.clone(),
        amount_text: format!("{:.2}", item.amount),
        pinned_text: format!("{}", item.pinned),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_pending_nonce_estimate_views(items: &[PendingNonceEstimate]) -> Vec<PendingNonceEstimateView> {
    let mut views: Vec<PendingNonceEstimateView> = Vec::new();
    for item in items.iter() {
        views.push(to_pending_nonce_estimate_view(item));
    }
    views
}

pub fn sum_pending_nonce_estimate_view(view: &PendingNonceEstimateView) -> Result<f64, String> {
    let parsed_window_days = view
        .window_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    Ok(parsed_window_days as f64 + parsed_margin + parsed_weight)
}
