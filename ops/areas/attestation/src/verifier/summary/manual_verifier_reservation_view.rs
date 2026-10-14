use super::*;

/// How screens and exports show a manual verifier reservation.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualVerifierReservationView {
    pub code: String,
    pub title_text: String,
    pub capacity: String,
    pub segment: String,
    pub batch_size: String,
    pub status_label: String,
}

pub fn to_manual_verifier_reservation_view(item: &ManualVerifierReservation) -> ManualVerifierReservationView {
    ManualVerifierReservationView {
        code: item.code.clone(),
        title_text: item.title.clone(),
        capacity: item.capacity.to_string(),
        segment: item.segment.clone(),
        batch_size: item.batch_size.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_manual_verifier_reservation_views(items: &[ManualVerifierReservation]) -> Vec<ManualVerifierReservationView> {
    let mut views: Vec<ManualVerifierReservationView> = Vec::new();
    for item in items.iter() {
        views.push(to_manual_verifier_reservation_view(item));
    }
    views
}

pub fn sum_manual_verifier_reservation_view(view: &ManualVerifierReservationView) -> Result<f64, String> {
    let parsed_capacity = view
        .capacity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    let parsed_batch_size = view
        .batch_size
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    Ok(parsed_capacity as f64 + (parsed_batch_size as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_manual_verifier_reservation("code-645", "title-110");
        let view = to_manual_verifier_reservation_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_manual_verifier_reservation_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
