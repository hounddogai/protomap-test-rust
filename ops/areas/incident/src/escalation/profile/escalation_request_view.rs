use super::*;

use acme_kit::text;

/// How screens and exports show a escalation request.
#[derive(Debug, Clone, PartialEq)]
pub struct EscalationRequestView {
    pub code: String,
    pub title: String,
    pub principal: String,
    pub unit: String,
    pub currency: String,
    pub ratio: String,
    pub window_days: String,
    pub port: String,
    pub starts_at: String,
    pub limit: String,
    pub status_label: String,
}

pub fn to_escalation_request_view(item: &EscalationRequest) -> EscalationRequestView {
    EscalationRequestView {
        code: text::truncate(&item.code, 80),
        title: item.title.clone(),
        principal: text::truncate(&item.principal, 120),
        unit: item.unit.clone(),
        currency: item.currency.clone(),
        ratio: format!("{:.2}", item.ratio),
        window_days: item.window_days.to_string(),
        port: item.port.to_string(),
        starts_at: item.starts_at.to_string(),
        limit: item.limit.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_escalation_request_views(items: &[EscalationRequest]) -> Vec<EscalationRequestView> {
    let mut views: Vec<EscalationRequestView> = Vec::new();
    for item in items.iter() {
        views.push(to_escalation_request_view(item));
    }
    views
}

pub fn sum_escalation_request_view(view: &EscalationRequestView) -> Result<f64, String> {
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    let parsed_window_days = view
        .window_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    Ok(parsed_ratio + (parsed_window_days as f64) + (parsed_port as f64))
}
