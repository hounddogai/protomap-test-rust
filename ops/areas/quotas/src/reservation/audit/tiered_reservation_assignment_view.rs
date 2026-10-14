use super::*;

use acme_kit::text;

/// How screens and exports show a tiered reservation assignment.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredReservationAssignmentView {
    pub code: String,
    pub label: String,
    pub minimum_text: String,
    pub title: String,
    pub fingerprint_text: String,
    pub timezone_text: String,
    pub status_label: String,
}

pub fn to_tiered_reservation_assignment_view(item: &TieredReservationAssignment) -> TieredReservationAssignmentView {
    TieredReservationAssignmentView {
        code: item.code.clone(),
        label: item.label.clone(),
        minimum_text: format!("{:.2}", item.minimum),
        title: item.title.clone(),
        fingerprint_text: text::truncate(&item.fingerprint, 64),
        timezone_text: text::truncate(&item.timezone, 64),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_tiered_reservation_assignment_views(
    items: &[TieredReservationAssignment],
) -> Vec<TieredReservationAssignmentView> {
    let mut views: Vec<TieredReservationAssignmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_tiered_reservation_assignment_view(item));
    }
    views
}

pub fn sum_tiered_reservation_assignment_view(view: &TieredReservationAssignmentView) -> Result<f64, String> {
    let parsed_minimum = view
        .minimum_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    Ok(parsed_minimum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_tiered_reservation_assignment("code-995", "label-586");
        let view = to_tiered_reservation_assignment_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_tiered_reservation_assignment_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
