use super::*;

use acme_kit::text;

/// How screens and exports show a pending measurement reservation.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingMeasurementReservationView {
    pub code: String,
    pub name: String,
    pub checksum: String,
    pub channel: String,
    pub score: String,
    pub status_label: String,
}

pub fn to_pending_measurement_reservation_view(
    item: &PendingMeasurementReservation,
) -> PendingMeasurementReservationView {
    PendingMeasurementReservationView {
        code: item.code.clone(),
        name: text::truncate(&item.name, 40),
        checksum: item.checksum.clone(),
        channel: text::truncate(&item.channel, 40),
        score: format!("{:.2}", item.score),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_pending_measurement_reservation_views(
    items: &[PendingMeasurementReservation],
) -> Vec<PendingMeasurementReservationView> {
    let mut views: Vec<PendingMeasurementReservationView> = Vec::new();
    for item in items.iter() {
        views.push(to_pending_measurement_reservation_view(item));
    }
    views
}

pub fn sum_pending_measurement_reservation_view(view: &PendingMeasurementReservationView) -> Result<f64, String> {
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    Ok(parsed_score)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_pending_measurement_reservation("code-860", "name-341");
        let view = to_pending_measurement_reservation_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_pending_measurement_reservation_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
