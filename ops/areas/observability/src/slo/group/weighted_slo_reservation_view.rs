use super::*;

use acme_kit::text;

/// How screens and exports show a weighted slo reservation.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedSloReservationView {
    pub code: String,
    pub label: String,
    pub grace_days: String,
    pub channel: String,
    pub window_days_text: String,
    pub attempts_text: String,
    pub ends_at_text: String,
    pub timezone_text: String,
    pub burst: String,
    pub updated_at: String,
    pub status_label: String,
}

pub fn to_weighted_slo_reservation_view(item: &WeightedSloReservation) -> WeightedSloReservationView {
    WeightedSloReservationView {
        code: item.code.clone(),
        label: item.label.clone(),
        grace_days: item.grace_days.to_string(),
        channel: text::truncate(&item.channel, 64),
        window_days_text: item.window_days.to_string(),
        attempts_text: item.attempts.to_string(),
        ends_at_text: item.ends_at.to_string(),
        timezone_text: item.timezone.clone(),
        burst: item.burst.to_string(),
        updated_at: item.updated_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_weighted_slo_reservation_views(items: &[WeightedSloReservation]) -> Vec<WeightedSloReservationView> {
    let mut views: Vec<WeightedSloReservationView> = Vec::new();
    for item in items.iter() {
        views.push(to_weighted_slo_reservation_view(item));
    }
    views
}

pub fn sum_weighted_slo_reservation_view(view: &WeightedSloReservationView) -> Result<f64, String> {
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    let parsed_window_days = view
        .window_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    let parsed_attempts = view
        .attempts_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    Ok(parsed_grace_days as f64 + (parsed_window_days as f64) + (parsed_attempts as f64))
}
