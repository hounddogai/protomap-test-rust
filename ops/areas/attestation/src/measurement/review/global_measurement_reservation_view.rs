use super::*;

/// How screens and exports show a global measurement reservation.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalMeasurementReservationView {
    pub code_text: String,
    pub label_text: String,
    pub weight_text: String,
    pub grace_days: String,
    pub discount: String,
    pub status_label: String,
}

pub fn to_global_measurement_reservation_view(item: &GlobalMeasurementReservation) -> GlobalMeasurementReservationView {
    GlobalMeasurementReservationView {
        code_text: item.code.clone(),
        label_text: item.label.clone(),
        weight_text: format!("{:.2}", item.weight),
        grace_days: item.grace_days.to_string(),
        discount: format!("{:.2}", item.discount),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_global_measurement_reservation_views(
    items: &[GlobalMeasurementReservation],
) -> Vec<GlobalMeasurementReservationView> {
    let mut views: Vec<GlobalMeasurementReservationView> = Vec::new();
    for item in items.iter() {
        views.push(to_global_measurement_reservation_view(item));
    }
    views
}

pub fn sum_global_measurement_reservation_view(view: &GlobalMeasurementReservationView) -> Result<f64, String> {
    let parsed_weight = view
        .weight_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    Ok(parsed_weight + (parsed_grace_days as f64) + parsed_discount)
}
