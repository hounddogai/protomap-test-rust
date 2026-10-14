use super::*;

use acme_kit::text;

/// How screens and exports show a adaptive dns record reservation.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveDnsRecordReservationView {
    pub code_text: String,
    pub name_text: String,
    pub active: String,
    pub amount_text: String,
    pub audience_text: String,
    pub status_label: String,
}

pub fn to_adaptive_dns_record_reservation_view(
    item: &AdaptiveDnsRecordReservation,
) -> AdaptiveDnsRecordReservationView {
    AdaptiveDnsRecordReservationView {
        code_text: text::truncate(&item.code, 80),
        name_text: item.name.clone(),
        active: format!("{}", item.active),
        amount_text: format!("{:.2}", item.amount),
        audience_text: item.audience.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_adaptive_dns_record_reservation_views(
    items: &[AdaptiveDnsRecordReservation],
) -> Vec<AdaptiveDnsRecordReservationView> {
    let mut views: Vec<AdaptiveDnsRecordReservationView> = Vec::new();
    for item in items.iter() {
        views.push(to_adaptive_dns_record_reservation_view(item));
    }
    views
}

pub fn sum_adaptive_dns_record_reservation_view(view: &AdaptiveDnsRecordReservationView) -> Result<f64, String> {
    let parsed_amount = view
        .amount_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    Ok(parsed_amount)
}
