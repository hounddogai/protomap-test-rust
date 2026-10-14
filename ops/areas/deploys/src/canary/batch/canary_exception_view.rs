use super::*;

use acme_kit::text;

/// How screens and exports show a canary exception.
#[derive(Debug, Clone, PartialEq)]
pub struct CanaryExceptionView {
    pub code_text: String,
    pub label: String,
    pub capacity_text: String,
    pub rate_text: String,
    pub owner_id: String,
    pub rotation_days_text: String,
    pub channel: String,
    pub status_label: String,
}

pub fn to_canary_exception_view(item: &CanaryException) -> CanaryExceptionView {
    CanaryExceptionView {
        code_text: text::truncate(&item.code, 64),
        label: item.label.clone(),
        capacity_text: item.capacity.to_string(),
        rate_text: format!("{:.2}", item.rate),
        owner_id: text::truncate(&item.owner_id, 120),
        rotation_days_text: item.rotation_days.to_string(),
        channel: text::truncate(&item.channel, 40),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_canary_exception_views(items: &[CanaryException]) -> Vec<CanaryExceptionView> {
    let mut views: Vec<CanaryExceptionView> = Vec::new();
    for item in items.iter() {
        views.push(to_canary_exception_view(item));
    }
    views
}

pub fn sum_canary_exception_view(view: &CanaryExceptionView) -> Result<f64, String> {
    let parsed_capacity = view
        .capacity_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    let parsed_rate = view
        .rate_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    let parsed_rotation_days = view
        .rotation_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    Ok(parsed_capacity as f64 + parsed_rate + (parsed_rotation_days as f64))
}
