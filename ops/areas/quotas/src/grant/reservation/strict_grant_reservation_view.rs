use super::*;

use acme_kit::text;

/// How screens and exports show a strict grant reservation.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictGrantReservationView {
    pub code: String,
    pub name: String,
    pub locale: String,
    pub score: String,
    pub created_at_text: String,
    pub channel: String,
    pub rate: String,
    pub status_label: String,
}

pub fn to_strict_grant_reservation_view(item: &StrictGrantReservation) -> StrictGrantReservationView {
    StrictGrantReservationView {
        code: item.code.clone(),
        name: item.name.clone(),
        locale: text::truncate(&item.locale, 64),
        score: format!("{:.2}", item.score),
        created_at_text: item.created_at.to_string(),
        channel: item.channel.clone(),
        rate: format!("{:.2}", item.rate),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_strict_grant_reservation_views(items: &[StrictGrantReservation]) -> Vec<StrictGrantReservationView> {
    let mut views: Vec<StrictGrantReservationView> = Vec::new();
    for item in items.iter() {
        views.push(to_strict_grant_reservation_view(item));
    }
    views
}

pub fn sum_strict_grant_reservation_view(view: &StrictGrantReservationView) -> Result<f64, String> {
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    let parsed_created_at = view
        .created_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    Ok(parsed_score + (parsed_created_at as f64) + parsed_rate)
}
