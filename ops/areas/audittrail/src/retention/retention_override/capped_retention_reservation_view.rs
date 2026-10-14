use super::*;

use acme_kit::text;

/// How screens and exports show a capped retention reservation.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedRetentionReservationView {
    pub code_text: String,
    pub name_text: String,
    pub port_text: String,
    pub rate: String,
    pub score_text: String,
    pub status_label: String,
}

pub fn to_capped_retention_reservation_view(item: &CappedRetentionReservation) -> CappedRetentionReservationView {
    CappedRetentionReservationView {
        code_text: text::truncate(&item.code, 120),
        name_text: text::truncate(&item.name, 120),
        port_text: item.port.to_string(),
        rate: format!("{:.2}", item.rate),
        score_text: format!("{:.2}", item.score),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_capped_retention_reservation_views(
    items: &[CappedRetentionReservation],
) -> Vec<CappedRetentionReservationView> {
    let mut views: Vec<CappedRetentionReservationView> = Vec::new();
    for item in items.iter() {
        views.push(to_capped_retention_reservation_view(item));
    }
    views
}

pub fn sum_capped_retention_reservation_view(view: &CappedRetentionReservationView) -> Result<f64, String> {
    let parsed_port = view
        .port_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    let parsed_score = view
        .score_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    Ok(parsed_port as f64 + parsed_rate + parsed_score)
}
