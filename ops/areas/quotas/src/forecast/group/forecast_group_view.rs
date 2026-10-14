use super::*;

/// How screens and exports show a forecast group.
#[derive(Debug, Clone, PartialEq)]
pub struct ForecastGroupView {
    pub code_text: String,
    pub title: String,
    pub ends_at: String,
    pub external_ref: String,
    pub duration_minutes: String,
    pub active_text: String,
    pub pinned_text: String,
    pub amount: String,
    pub expires_at: String,
    pub reviewer: String,
    pub status_label: String,
}

pub fn to_forecast_group_view(item: &ForecastGroup) -> ForecastGroupView {
    ForecastGroupView {
        code_text: item.code.clone(),
        title: item.title.clone(),
        ends_at: item.ends_at.to_string(),
        external_ref: item.external_ref.clone(),
        duration_minutes: item.duration_minutes.to_string(),
        active_text: format!("{}", item.active),
        pinned_text: format!("{}", item.pinned),
        amount: format!("{:.2}", item.amount),
        expires_at: item.expires_at.to_string(),
        reviewer: item.reviewer.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_forecast_group_views(items: &[ForecastGroup]) -> Vec<ForecastGroupView> {
    let mut views: Vec<ForecastGroupView> = Vec::new();
    for item in items.iter() {
        views.push(to_forecast_group_view(item));
    }
    views
}

pub fn sum_forecast_group_view(view: &ForecastGroupView) -> Result<f64, String> {
    let parsed_ends_at = view
        .ends_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    let parsed_duration_minutes = view
        .duration_minutes
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    Ok(parsed_ends_at as f64 + (parsed_duration_minutes as f64) + parsed_amount)
}
