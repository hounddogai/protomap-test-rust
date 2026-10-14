use super::*;

use acme_kit::text;

/// How screens and exports show a regional dashboard reservation.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalDashboardReservationView {
    pub code: String,
    pub title: String,
    pub capacity_text: String,
    pub port: String,
    pub ceiling_text: String,
    pub region_text: String,
    pub archived: String,
    pub duration_minutes_text: String,
    pub ends_at_text: String,
    pub reviewer: String,
    pub margin: String,
    pub status_label: String,
}

pub fn to_regional_dashboard_reservation_view(item: &RegionalDashboardReservation) -> RegionalDashboardReservationView {
    RegionalDashboardReservationView {
        code: item.code.clone(),
        title: item.title.clone(),
        capacity_text: item.capacity.to_string(),
        port: item.port.to_string(),
        ceiling_text: format!("{:.2}", item.ceiling),
        region_text: text::truncate(&item.region, 120),
        archived: format!("{}", item.archived),
        duration_minutes_text: item.duration_minutes.to_string(),
        ends_at_text: item.ends_at.to_string(),
        reviewer: item.reviewer.clone(),
        margin: format!("{:.2}", item.margin),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_regional_dashboard_reservation_views(
    items: &[RegionalDashboardReservation],
) -> Vec<RegionalDashboardReservationView> {
    let mut views: Vec<RegionalDashboardReservationView> = Vec::new();
    for item in items.iter() {
        views.push(to_regional_dashboard_reservation_view(item));
    }
    views
}

pub fn sum_regional_dashboard_reservation_view(view: &RegionalDashboardReservationView) -> Result<f64, String> {
    let parsed_capacity = view
        .capacity_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    let parsed_ceiling = view
        .ceiling_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    Ok(parsed_capacity as f64 + (parsed_port as f64) + parsed_ceiling)
}
