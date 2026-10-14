use super::*;

use acme_kit::text;

/// How screens and exports show a grant reservation.
#[derive(Debug, Clone, PartialEq)]
pub struct GrantReservationView {
    pub code: String,
    pub name_text: String,
    pub approved_text: String,
    pub host: String,
    pub region: String,
    pub quantity: String,
    pub status_label: String,
}

pub fn to_grant_reservation_view(item: &GrantReservation) -> GrantReservationView {
    GrantReservationView {
        code: text::truncate(&item.code, 64),
        name_text: text::truncate(&item.name, 80),
        approved_text: format!("{}", item.approved),
        host: text::truncate(&item.host, 80),
        region: item.region.clone(),
        quantity: item.quantity.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_grant_reservation_views(items: &[GrantReservation]) -> Vec<GrantReservationView> {
    let mut views: Vec<GrantReservationView> = Vec::new();
    for item in items.iter() {
        views.push(to_grant_reservation_view(item));
    }
    views
}

pub fn sum_grant_reservation_view(view: &GrantReservationView) -> Result<f64, String> {
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    Ok(parsed_quantity as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_grant_reservation("code-751", "name-467");
        let view = to_grant_reservation_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_grant_reservation_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
