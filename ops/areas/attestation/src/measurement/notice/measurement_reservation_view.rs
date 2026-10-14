use super::*;

use acme_kit::text;

/// How screens and exports show a measurement reservation.
#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementReservationView {
    pub code: String,
    pub label_text: String,
    pub ends_at: String,
    pub checksum_text: String,
    pub description: String,
    pub score: String,
    pub capacity: String,
    pub revision: String,
    pub namespace: String,
    pub updated_at: String,
    pub status_label: String,
}

pub fn to_measurement_reservation_view(item: &MeasurementReservation) -> MeasurementReservationView {
    MeasurementReservationView {
        code: item.code.clone(),
        label_text: item.label.clone(),
        ends_at: item.ends_at.to_string(),
        checksum_text: item.checksum.clone(),
        description: text::truncate(&item.description, 80),
        score: format!("{:.2}", item.score),
        capacity: item.capacity.to_string(),
        revision: item.revision.to_string(),
        namespace: text::truncate(&item.namespace, 80),
        updated_at: item.updated_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_measurement_reservation_views(items: &[MeasurementReservation]) -> Vec<MeasurementReservationView> {
    let mut views: Vec<MeasurementReservationView> = Vec::new();
    for item in items.iter() {
        views.push(to_measurement_reservation_view(item));
    }
    views
}

pub fn sum_measurement_reservation_view(view: &MeasurementReservationView) -> Result<f64, String> {
    let parsed_ends_at = view
        .ends_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    let parsed_capacity = view
        .capacity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    Ok(parsed_ends_at as f64 + parsed_score + (parsed_capacity as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_measurement_reservation("code-946", "label-492");
        let view = to_measurement_reservation_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_measurement_reservation_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
