use super::*;

use acme_kit::text;

/// How screens and exports show a primary dashboard reservation.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryDashboardReservationView {
    pub code: String,
    pub title_text: String,
    pub external_ref: String,
    pub unit: String,
    pub starts_at_text: String,
    pub audience_text: String,
    pub batch_size: String,
    pub status_label: String,
}

pub fn to_primary_dashboard_reservation_view(item: &PrimaryDashboardReservation) -> PrimaryDashboardReservationView {
    PrimaryDashboardReservationView {
        code: text::truncate(&item.code, 40),
        title_text: text::truncate(&item.title, 80),
        external_ref: text::truncate(&item.external_ref, 64),
        unit: text::truncate(&item.unit, 40),
        starts_at_text: item.starts_at.to_string(),
        audience_text: item.audience.clone(),
        batch_size: item.batch_size.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_primary_dashboard_reservation_views(
    items: &[PrimaryDashboardReservation],
) -> Vec<PrimaryDashboardReservationView> {
    let mut views: Vec<PrimaryDashboardReservationView> = Vec::new();
    for item in items.iter() {
        views.push(to_primary_dashboard_reservation_view(item));
    }
    views
}

pub fn sum_primary_dashboard_reservation_view(view: &PrimaryDashboardReservationView) -> Result<f64, String> {
    let parsed_starts_at = view
        .starts_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    let parsed_batch_size = view
        .batch_size
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    Ok(parsed_starts_at as f64 + (parsed_batch_size as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_primary_dashboard_reservation("code-521", "title-613");
        let view = to_primary_dashboard_reservation_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_primary_dashboard_reservation_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
