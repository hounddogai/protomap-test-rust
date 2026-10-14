use super::*;

use acme_kit::text;

/// How screens and exports show a suspension reservation.
#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionReservationView {
    pub code_text: String,
    pub label_text: String,
    pub attempts: String,
    pub title: String,
    pub phone: String,
    pub status_label: String,
}

pub fn to_suspension_reservation_view(item: &SuspensionReservation) -> SuspensionReservationView {
    SuspensionReservationView {
        code_text: item.code.clone(),
        label_text: text::truncate(&item.label, 120),
        attempts: item.attempts.to_string(),
        title: item.title.clone(),
        phone: item.phone.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_suspension_reservation_views(items: &[SuspensionReservation]) -> Vec<SuspensionReservationView> {
    let mut views: Vec<SuspensionReservationView> = Vec::new();
    for item in items.iter() {
        views.push(to_suspension_reservation_view(item));
    }
    views
}

pub fn sum_suspension_reservation_view(view: &SuspensionReservationView) -> Result<f64, String> {
    let parsed_attempts = view
        .attempts
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    Ok(parsed_attempts as f64)
}
