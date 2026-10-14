use super::*;

use acme_kit::text;

/// How screens and exports show a pending limit reservation.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingLimitReservationView {
    pub code: String,
    pub label_text: String,
    pub archived: String,
    pub namespace_text: String,
    pub quota_units: String,
    pub rate: String,
    pub status_label: String,
}

pub fn to_pending_limit_reservation_view(item: &PendingLimitReservation) -> PendingLimitReservationView {
    PendingLimitReservationView {
        code: item.code.clone(),
        label_text: text::truncate(&item.label, 80),
        archived: format!("{}", item.archived),
        namespace_text: item.namespace.clone(),
        quota_units: item.quota_units.to_string(),
        rate: format!("{:.2}", item.rate),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_pending_limit_reservation_views(items: &[PendingLimitReservation]) -> Vec<PendingLimitReservationView> {
    let mut views: Vec<PendingLimitReservationView> = Vec::new();
    for item in items.iter() {
        views.push(to_pending_limit_reservation_view(item));
    }
    views
}

pub fn sum_pending_limit_reservation_view(view: &PendingLimitReservationView) -> Result<f64, String> {
    let parsed_quota_units = view
        .quota_units
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    Ok(parsed_quota_units as f64 + parsed_rate)
}
