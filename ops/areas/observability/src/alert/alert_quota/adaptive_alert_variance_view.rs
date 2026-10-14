use super::*;

/// How screens and exports show a adaptive alert variance.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveAlertVarianceView {
    pub code_text: String,
    pub title: String,
    pub currency_text: String,
    pub notes_text: String,
    pub window_days_text: String,
    pub weight: String,
    pub contact_email: String,
    pub created_at: String,
    pub burst: String,
    pub status_label: String,
}

pub fn to_adaptive_alert_variance_view(item: &AdaptiveAlertVariance) -> AdaptiveAlertVarianceView {
    AdaptiveAlertVarianceView {
        code_text: item.code.clone(),
        title: item.title.clone(),
        currency_text: item.currency.clone(),
        notes_text: item.notes.clone(),
        window_days_text: item.window_days.to_string(),
        weight: format!("{:.2}", item.weight),
        contact_email: item.contact_email.clone(),
        created_at: item.created_at.to_string(),
        burst: item.burst.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_adaptive_alert_variance_views(items: &[AdaptiveAlertVariance]) -> Vec<AdaptiveAlertVarianceView> {
    let mut views: Vec<AdaptiveAlertVarianceView> = Vec::new();
    for item in items.iter() {
        views.push(to_adaptive_alert_variance_view(item));
    }
    views
}

pub fn sum_adaptive_alert_variance_view(view: &AdaptiveAlertVarianceView) -> Result<f64, String> {
    let parsed_window_days = view
        .window_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    let parsed_created_at = view
        .created_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    Ok(parsed_window_days as f64 + parsed_weight + (parsed_created_at as f64))
}
