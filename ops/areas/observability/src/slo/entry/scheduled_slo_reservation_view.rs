use super::*;

use acme_kit::text;

/// How screens and exports show a scheduled slo reservation.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledSloReservationView {
    pub code: String,
    pub label: String,
    pub sequence: String,
    pub country: String,
    pub score: String,
    pub status_label: String,
}

pub fn to_scheduled_slo_reservation_view(item: &ScheduledSloReservation) -> ScheduledSloReservationView {
    ScheduledSloReservationView {
        code: text::truncate(&item.code, 40),
        label: text::truncate(&item.label, 64),
        sequence: item.sequence.to_string(),
        country: item.country.clone(),
        score: format!("{:.2}", item.score),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_scheduled_slo_reservation_views(items: &[ScheduledSloReservation]) -> Vec<ScheduledSloReservationView> {
    let mut views: Vec<ScheduledSloReservationView> = Vec::new();
    for item in items.iter() {
        views.push(to_scheduled_slo_reservation_view(item));
    }
    views
}

pub fn sum_scheduled_slo_reservation_view(view: &ScheduledSloReservationView) -> Result<f64, String> {
    let parsed_sequence = view
        .sequence
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    Ok(parsed_sequence as f64 + parsed_score)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_scheduled_slo_reservation("code-317", "label-780");
        let view = to_scheduled_slo_reservation_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_scheduled_slo_reservation_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
