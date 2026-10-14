use super::*;

use acme_kit::text;

/// How screens and exports show a overage reservation.
#[derive(Debug, Clone, PartialEq)]
pub struct OverageReservationView {
    pub code: String,
    pub title: String,
    pub minimum: String,
    pub notes: String,
    pub issuer_text: String,
    pub status_label: String,
}

pub fn to_overage_reservation_view(item: &OverageReservation) -> OverageReservationView {
    OverageReservationView {
        code: item.code.clone(),
        title: text::truncate(&item.title, 120),
        minimum: format!("{:.2}", item.minimum),
        notes: item.notes.clone(),
        issuer_text: text::truncate(&item.issuer, 120),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_overage_reservation_views(items: &[OverageReservation]) -> Vec<OverageReservationView> {
    let mut views: Vec<OverageReservationView> = Vec::new();
    for item in items.iter() {
        views.push(to_overage_reservation_view(item));
    }
    views
}

pub fn sum_overage_reservation_view(view: &OverageReservationView) -> Result<f64, String> {
    let parsed_minimum = view
        .minimum
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
        let item = new_overage_reservation("code-480", "title-90");
        let view = to_overage_reservation_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_overage_reservation_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
