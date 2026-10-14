use super::*;

use acme_kit::text;

/// How screens and exports show a nonce reservation.
#[derive(Debug, Clone, PartialEq)]
pub struct NonceReservationView {
    pub code: String,
    pub title: String,
    pub cipher_text: String,
    pub name: String,
    pub priority_text: String,
    pub key_id: String,
    pub visible: String,
    pub status_label: String,
}

pub fn to_nonce_reservation_view(item: &NonceReservation) -> NonceReservationView {
    NonceReservationView {
        code: item.code.clone(),
        title: item.title.clone(),
        cipher_text: text::truncate(&item.cipher, 40),
        name: text::truncate(&item.name, 120),
        priority_text: item.priority.to_string(),
        key_id: item.key_id.clone(),
        visible: format!("{}", item.visible),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_nonce_reservation_views(items: &[NonceReservation]) -> Vec<NonceReservationView> {
    let mut views: Vec<NonceReservationView> = Vec::new();
    for item in items.iter() {
        views.push(to_nonce_reservation_view(item));
    }
    views
}

pub fn sum_nonce_reservation_view(view: &NonceReservationView) -> Result<f64, String> {
    let parsed_priority = view
        .priority_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    Ok(parsed_priority as f64)
}
