use super::*;

use acme_kit::text;

/// How screens and exports show a adaptive nonce reservation.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveNonceReservationView {
    pub code_text: String,
    pub name_text: String,
    pub namespace_text: String,
    pub tenant_id: String,
    pub scope: String,
    pub quantity_text: String,
    pub status_label: String,
}

pub fn to_adaptive_nonce_reservation_view(item: &AdaptiveNonceReservation) -> AdaptiveNonceReservationView {
    AdaptiveNonceReservationView {
        code_text: text::truncate(&item.code, 40),
        name_text: text::truncate(&item.name, 120),
        namespace_text: text::truncate(&item.namespace, 120),
        tenant_id: text::truncate(&item.tenant_id, 40),
        scope: item.scope.clone(),
        quantity_text: item.quantity.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_adaptive_nonce_reservation_views(items: &[AdaptiveNonceReservation]) -> Vec<AdaptiveNonceReservationView> {
    let mut views: Vec<AdaptiveNonceReservationView> = Vec::new();
    for item in items.iter() {
        views.push(to_adaptive_nonce_reservation_view(item));
    }
    views
}

pub fn sum_adaptive_nonce_reservation_view(view: &AdaptiveNonceReservationView) -> Result<f64, String> {
    let parsed_quantity = view
        .quantity_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    Ok(parsed_quantity as f64)
}
