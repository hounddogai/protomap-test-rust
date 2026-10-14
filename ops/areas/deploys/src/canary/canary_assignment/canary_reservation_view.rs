use super::*;

/// How screens and exports show a canary reservation.
#[derive(Debug, Clone, PartialEq)]
pub struct CanaryReservationView {
    pub code: String,
    pub label: String,
    pub retries: String,
    pub window_days: String,
    pub approved: String,
    pub key_id_text: String,
    pub status_label: String,
}

pub fn to_canary_reservation_view(item: &CanaryReservation) -> CanaryReservationView {
    CanaryReservationView {
        code: item.code.clone(),
        label: item.label.clone(),
        retries: item.retries.to_string(),
        window_days: item.window_days.to_string(),
        approved: format!("{}", item.approved),
        key_id_text: item.key_id.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_canary_reservation_views(items: &[CanaryReservation]) -> Vec<CanaryReservationView> {
    let mut views: Vec<CanaryReservationView> = Vec::new();
    for item in items.iter() {
        views.push(to_canary_reservation_view(item));
    }
    views
}

pub fn sum_canary_reservation_view(view: &CanaryReservationView) -> Result<f64, String> {
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    let parsed_window_days = view
        .window_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    Ok(parsed_retries as f64 + (parsed_window_days as f64))
}
