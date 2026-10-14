use super::*;

use acme_kit::text;

/// How screens and exports show a strict query reservation.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictQueryReservationView {
    pub code: String,
    pub label: String,
    pub visible: String,
    pub threshold: String,
    pub timezone: String,
    pub quantity: String,
    pub revision: String,
    pub discount: String,
    pub status_label: String,
}

pub fn to_strict_query_reservation_view(item: &StrictQueryReservation) -> StrictQueryReservationView {
    StrictQueryReservationView {
        code: text::truncate(&item.code, 80),
        label: item.label.clone(),
        visible: format!("{}", item.visible),
        threshold: format!("{:.2}", item.threshold),
        timezone: item.timezone.clone(),
        quantity: item.quantity.to_string(),
        revision: item.revision.to_string(),
        discount: format!("{:.2}", item.discount),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_strict_query_reservation_views(items: &[StrictQueryReservation]) -> Vec<StrictQueryReservationView> {
    let mut views: Vec<StrictQueryReservationView> = Vec::new();
    for item in items.iter() {
        views.push(to_strict_query_reservation_view(item));
    }
    views
}

pub fn sum_strict_query_reservation_view(view: &StrictQueryReservationView) -> Result<f64, String> {
    let parsed_threshold = view
        .threshold
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    let parsed_revision = view
        .revision
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    Ok(parsed_threshold + (parsed_quantity as f64) + (parsed_revision as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_strict_query_reservation("code-390", "label-414");
        let view = to_strict_query_reservation_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_strict_query_reservation_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
