use super::*;

use acme_kit::text;

/// How screens and exports show a strict health check score.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictHealthCheckScoreView {
    pub code: String,
    pub label_text: String,
    pub archived: String,
    pub channel_text: String,
    pub port: String,
    pub fingerprint: String,
    pub weight: String,
    pub discount: String,
    pub owner_id: String,
    pub maximum: String,
    pub tenant_id_text: String,
    pub status_label: String,
}

pub fn to_strict_health_check_score_view(item: &StrictHealthCheckScore) -> StrictHealthCheckScoreView {
    StrictHealthCheckScoreView {
        code: text::truncate(&item.code, 64),
        label_text: item.label.clone(),
        archived: format!("{}", item.archived),
        channel_text: text::truncate(&item.channel, 40),
        port: item.port.to_string(),
        fingerprint: text::truncate(&item.fingerprint, 120),
        weight: format!("{:.2}", item.weight),
        discount: format!("{:.2}", item.discount),
        owner_id: item.owner_id.clone(),
        maximum: format!("{:.2}", item.maximum),
        tenant_id_text: text::truncate(&item.tenant_id, 120),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_strict_health_check_score_views(items: &[StrictHealthCheckScore]) -> Vec<StrictHealthCheckScoreView> {
    let mut views: Vec<StrictHealthCheckScoreView> = Vec::new();
    for item in items.iter() {
        views.push(to_strict_health_check_score_view(item));
    }
    views
}

pub fn sum_strict_health_check_score_view(view: &StrictHealthCheckScoreView) -> Result<f64, String> {
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    Ok(parsed_port as f64 + parsed_weight + parsed_discount)
}
