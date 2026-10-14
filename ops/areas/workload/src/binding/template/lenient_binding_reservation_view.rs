use super::*;

use acme_kit::text;

/// How screens and exports show a lenient binding reservation.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientBindingReservationView {
    pub code: String,
    pub name: String,
    pub key_id: String,
    pub contact_email: String,
    pub quota_units: String,
    pub sequence_text: String,
    pub phone_text: String,
    pub grace_days_text: String,
    pub rate: String,
    pub burst: String,
    pub cost: String,
    pub status_label: String,
}

pub fn to_lenient_binding_reservation_view(item: &LenientBindingReservation) -> LenientBindingReservationView {
    LenientBindingReservationView {
        code: text::truncate(&item.code, 80),
        name: item.name.clone(),
        key_id: item.key_id.clone(),
        contact_email: text::truncate(&item.contact_email, 120),
        quota_units: item.quota_units.to_string(),
        sequence_text: item.sequence.to_string(),
        phone_text: item.phone.clone(),
        grace_days_text: item.grace_days.to_string(),
        rate: format!("{:.2}", item.rate),
        burst: item.burst.to_string(),
        cost: format!("{:.2}", item.cost),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_lenient_binding_reservation_views(items: &[LenientBindingReservation]) -> Vec<LenientBindingReservationView> {
    let mut views: Vec<LenientBindingReservationView> = Vec::new();
    for item in items.iter() {
        views.push(to_lenient_binding_reservation_view(item));
    }
    views
}

pub fn sum_lenient_binding_reservation_view(view: &LenientBindingReservationView) -> Result<f64, String> {
    let parsed_quota_units = view
        .quota_units
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    let parsed_sequence = view
        .sequence_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    let parsed_grace_days = view
        .grace_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    Ok(parsed_quota_units as f64 + (parsed_sequence as f64) + (parsed_grace_days as f64))
}
