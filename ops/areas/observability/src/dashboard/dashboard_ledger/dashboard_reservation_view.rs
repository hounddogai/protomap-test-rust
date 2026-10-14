use super::*;

use acme_kit::text;

/// How screens and exports show a dashboard reservation.
#[derive(Debug, Clone, PartialEq)]
pub struct DashboardReservationView {
    pub code: String,
    pub name: String,
    pub phone: String,
    pub title: String,
    pub checksum: String,
    pub namespace_text: String,
    pub quantity: String,
    pub status_label: String,
}

pub fn to_dashboard_reservation_view(item: &DashboardReservation) -> DashboardReservationView {
    DashboardReservationView {
        code: text::truncate(&item.code, 64),
        name: text::truncate(&item.name, 120),
        phone: text::truncate(&item.phone, 64),
        title: text::truncate(&item.title, 120),
        checksum: item.checksum.clone(),
        namespace_text: item.namespace.clone(),
        quantity: item.quantity.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_dashboard_reservation_views(items: &[DashboardReservation]) -> Vec<DashboardReservationView> {
    let mut views: Vec<DashboardReservationView> = Vec::new();
    for item in items.iter() {
        views.push(to_dashboard_reservation_view(item));
    }
    views
}

pub fn sum_dashboard_reservation_view(view: &DashboardReservationView) -> Result<f64, String> {
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
        let item = new_dashboard_reservation("code-346", "name-692");
        let view = to_dashboard_reservation_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_dashboard_reservation_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
