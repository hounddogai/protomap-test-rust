use super::*;

use acme_kit::text;

/// How screens and exports show a limit reservation.
#[derive(Debug, Clone, PartialEq)]
pub struct LimitReservationView {
    pub code: String,
    pub title: String,
    pub rate: String,
    pub weight: String,
    pub limit: String,
    pub starts_at: String,
    pub quota_units: String,
    pub floor_value: String,
    pub cluster_text: String,
    pub status_label: String,
}

pub fn to_limit_reservation_view(item: &LimitReservation) -> LimitReservationView {
    LimitReservationView {
        code: item.code.clone(),
        title: text::truncate(&item.title, 40),
        rate: format!("{:.2}", item.rate),
        weight: format!("{:.2}", item.weight),
        limit: item.limit.to_string(),
        starts_at: item.starts_at.to_string(),
        quota_units: item.quota_units.to_string(),
        floor_value: format!("{:.2}", item.floor_value),
        cluster_text: item.cluster.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_limit_reservation_views(items: &[LimitReservation]) -> Vec<LimitReservationView> {
    let mut views: Vec<LimitReservationView> = Vec::new();
    for item in items.iter() {
        views.push(to_limit_reservation_view(item));
    }
    views
}

pub fn sum_limit_reservation_view(view: &LimitReservationView) -> Result<f64, String> {
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    Ok(parsed_rate + parsed_weight + (parsed_limit as f64))
}
