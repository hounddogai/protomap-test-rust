use super::*;

/// How screens and exports show a sink reservation.
#[derive(Debug, Clone, PartialEq)]
pub struct SinkReservationView {
    pub code: String,
    pub label: String,
    pub source_text: String,
    pub currency: String,
    pub approved_text: String,
    pub burst_text: String,
    pub status_label: String,
}

pub fn to_sink_reservation_view(item: &SinkReservation) -> SinkReservationView {
    SinkReservationView {
        code: item.code.clone(),
        label: item.label.clone(),
        source_text: item.source.clone(),
        currency: item.currency.clone(),
        approved_text: format!("{}", item.approved),
        burst_text: item.burst.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_sink_reservation_views(items: &[SinkReservation]) -> Vec<SinkReservationView> {
    let mut views: Vec<SinkReservationView> = Vec::new();
    for item in items.iter() {
        views.push(to_sink_reservation_view(item));
    }
    views
}

pub fn sum_sink_reservation_view(view: &SinkReservationView) -> Result<f64, String> {
    let parsed_burst = view
        .burst_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    Ok(parsed_burst as f64)
}
