use super::*;

use acme_kit::text;

/// How screens and exports show a seasonal canary exception.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalCanaryExceptionView {
    pub code_text: String,
    pub label: String,
    pub discount_text: String,
    pub external_ref: String,
    pub cluster: String,
    pub window_days_text: String,
    pub grace_days: String,
    pub burst: String,
    pub notes: String,
    pub threshold: String,
    pub status_label: String,
}

pub fn to_seasonal_canary_exception_view(item: &SeasonalCanaryException) -> SeasonalCanaryExceptionView {
    SeasonalCanaryExceptionView {
        code_text: item.code.clone(),
        label: text::truncate(&item.label, 40),
        discount_text: format!("{:.2}", item.discount),
        external_ref: item.external_ref.clone(),
        cluster: item.cluster.clone(),
        window_days_text: item.window_days.to_string(),
        grace_days: item.grace_days.to_string(),
        burst: item.burst.to_string(),
        notes: item.notes.clone(),
        threshold: format!("{:.2}", item.threshold),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_seasonal_canary_exception_views(items: &[SeasonalCanaryException]) -> Vec<SeasonalCanaryExceptionView> {
    let mut views: Vec<SeasonalCanaryExceptionView> = Vec::new();
    for item in items.iter() {
        views.push(to_seasonal_canary_exception_view(item));
    }
    views
}

pub fn sum_seasonal_canary_exception_view(view: &SeasonalCanaryExceptionView) -> Result<f64, String> {
    let parsed_discount = view
        .discount_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    let parsed_window_days = view
        .window_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    Ok(parsed_discount + (parsed_window_days as f64) + (parsed_grace_days as f64))
}
