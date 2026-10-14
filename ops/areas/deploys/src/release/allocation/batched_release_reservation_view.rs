use super::*;

use acme_kit::text;

/// How screens and exports show a batched release reservation.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedReleaseReservationView {
    pub code: String,
    pub name: String,
    pub country: String,
    pub cost_text: String,
    pub description: String,
    pub status_label: String,
}

pub fn to_batched_release_reservation_view(item: &BatchedReleaseReservation) -> BatchedReleaseReservationView {
    BatchedReleaseReservationView {
        code: item.code.clone(),
        name: text::truncate(&item.name, 120),
        country: item.country.clone(),
        cost_text: format!("{:.2}", item.cost),
        description: item.description.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_batched_release_reservation_views(items: &[BatchedReleaseReservation]) -> Vec<BatchedReleaseReservationView> {
    let mut views: Vec<BatchedReleaseReservationView> = Vec::new();
    for item in items.iter() {
        views.push(to_batched_release_reservation_view(item));
    }
    views
}

pub fn sum_batched_release_reservation_view(view: &BatchedReleaseReservationView) -> Result<f64, String> {
    let parsed_cost = view
        .cost_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    Ok(parsed_cost)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_batched_release_reservation("code-297", "name-265");
        let view = to_batched_release_reservation_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_batched_release_reservation_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
