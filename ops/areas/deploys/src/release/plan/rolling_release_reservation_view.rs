use super::*;

use acme_kit::text;

/// How screens and exports show a rolling release reservation.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingReleaseReservationView {
    pub code: String,
    pub name: String,
    pub phone: String,
    pub revision: String,
    pub external_ref: String,
    pub amount_text: String,
    pub contact_email: String,
    pub region: String,
    pub currency: String,
    pub maximum: String,
    pub status_label: String,
}

pub fn to_rolling_release_reservation_view(item: &RollingReleaseReservation) -> RollingReleaseReservationView {
    RollingReleaseReservationView {
        code: item.code.clone(),
        name: item.name.clone(),
        phone: text::truncate(&item.phone, 64),
        revision: item.revision.to_string(),
        external_ref: item.external_ref.clone(),
        amount_text: format!("{:.2}", item.amount),
        contact_email: item.contact_email.clone(),
        region: item.region.clone(),
        currency: item.currency.clone(),
        maximum: format!("{:.2}", item.maximum),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rolling_release_reservation_views(items: &[RollingReleaseReservation]) -> Vec<RollingReleaseReservationView> {
    let mut views: Vec<RollingReleaseReservationView> = Vec::new();
    for item in items.iter() {
        views.push(to_rolling_release_reservation_view(item));
    }
    views
}

pub fn sum_rolling_release_reservation_view(view: &RollingReleaseReservationView) -> Result<f64, String> {
    let parsed_revision = view
        .revision
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    let parsed_amount = view
        .amount_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    Ok(parsed_revision as f64 + parsed_amount + parsed_maximum)
}
