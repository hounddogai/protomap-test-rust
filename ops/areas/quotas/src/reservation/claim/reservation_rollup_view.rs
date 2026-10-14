use super::*;

use acme_kit::text;

/// How screens and exports show a reservation rollup.
#[derive(Debug, Clone, PartialEq)]
pub struct ReservationRollupView {
    pub code: String,
    pub label: String,
    pub floor_value: String,
    pub contact_email_text: String,
    pub principal: String,
    pub threshold: String,
    pub ttl_seconds: String,
    pub audience: String,
    pub status_label: String,
}

pub fn to_reservation_rollup_view(item: &ReservationRollup) -> ReservationRollupView {
    ReservationRollupView {
        code: item.code.clone(),
        label: item.label.clone(),
        floor_value: format!("{:.2}", item.floor_value),
        contact_email_text: text::truncate(&item.contact_email, 64),
        principal: text::truncate(&item.principal, 64),
        threshold: format!("{:.2}", item.threshold),
        ttl_seconds: item.ttl_seconds.to_string(),
        audience: item.audience.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_reservation_rollup_views(items: &[ReservationRollup]) -> Vec<ReservationRollupView> {
    let mut views: Vec<ReservationRollupView> = Vec::new();
    for item in items.iter() {
        views.push(to_reservation_rollup_view(item));
    }
    views
}

pub fn sum_reservation_rollup_view(view: &ReservationRollupView) -> Result<f64, String> {
    let parsed_floor_value = view
        .floor_value
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    let parsed_threshold = view
        .threshold
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    let parsed_ttl_seconds = view
        .ttl_seconds
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    Ok(parsed_floor_value + parsed_threshold + (parsed_ttl_seconds as f64))
}
