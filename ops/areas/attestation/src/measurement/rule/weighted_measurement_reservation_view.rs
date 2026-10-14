use super::*;

/// How screens and exports show a weighted measurement reservation.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedMeasurementReservationView {
    pub code: String,
    pub title: String,
    pub ttl_seconds: String,
    pub country_text: String,
    pub key_id: String,
    pub cost: String,
    pub contact_email: String,
    pub rate: String,
    pub batch_size: String,
    pub status_label: String,
}

pub fn to_weighted_measurement_reservation_view(
    item: &WeightedMeasurementReservation,
) -> WeightedMeasurementReservationView {
    WeightedMeasurementReservationView {
        code: item.code.clone(),
        title: item.title.clone(),
        ttl_seconds: item.ttl_seconds.to_string(),
        country_text: item.country.clone(),
        key_id: item.key_id.clone(),
        cost: format!("{:.2}", item.cost),
        contact_email: item.contact_email.clone(),
        rate: format!("{:.2}", item.rate),
        batch_size: item.batch_size.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_weighted_measurement_reservation_views(
    items: &[WeightedMeasurementReservation],
) -> Vec<WeightedMeasurementReservationView> {
    let mut views: Vec<WeightedMeasurementReservationView> = Vec::new();
    for item in items.iter() {
        views.push(to_weighted_measurement_reservation_view(item));
    }
    views
}

pub fn sum_weighted_measurement_reservation_view(view: &WeightedMeasurementReservationView) -> Result<f64, String> {
    let parsed_ttl_seconds = view
        .ttl_seconds
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    let parsed_cost = view
        .cost
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    Ok(parsed_ttl_seconds as f64 + parsed_cost + parsed_rate)
}
