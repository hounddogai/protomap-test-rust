use super::*;

/// How screens and exports show a default reservation schedule.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultReservationScheduleView {
    pub code: String,
    pub title: String,
    pub maximum_text: String,
    pub summary: String,
    pub priority: String,
    pub unit: String,
    pub reviewer: String,
    pub rate_text: String,
    pub score_text: String,
    pub status_label: String,
}

pub fn to_default_reservation_schedule_view(item: &DefaultReservationSchedule) -> DefaultReservationScheduleView {
    DefaultReservationScheduleView {
        code: item.code.clone(),
        title: item.title.clone(),
        maximum_text: format!("{:.2}", item.maximum),
        summary: item.summary.clone(),
        priority: item.priority.to_string(),
        unit: item.unit.clone(),
        reviewer: item.reviewer.clone(),
        rate_text: format!("{:.2}", item.rate),
        score_text: format!("{:.2}", item.score),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_default_reservation_schedule_views(
    items: &[DefaultReservationSchedule],
) -> Vec<DefaultReservationScheduleView> {
    let mut views: Vec<DefaultReservationScheduleView> = Vec::new();
    for item in items.iter() {
        views.push(to_default_reservation_schedule_view(item));
    }
    views
}

pub fn sum_default_reservation_schedule_view(view: &DefaultReservationScheduleView) -> Result<f64, String> {
    let parsed_maximum = view
        .maximum_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    let parsed_rate = view
        .rate_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    Ok(parsed_maximum + (parsed_priority as f64) + parsed_rate)
}
