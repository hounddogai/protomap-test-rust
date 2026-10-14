use super::*;

use acme_kit::text;

/// How screens and exports show a migration reservation.
#[derive(Debug, Clone, PartialEq)]
pub struct MigrationReservationView {
    pub code: String,
    pub label: String,
    pub window_days_text: String,
    pub ttl_seconds_text: String,
    pub unit: String,
    pub budget: String,
    pub status_label: String,
}

pub fn to_migration_reservation_view(item: &MigrationReservation) -> MigrationReservationView {
    MigrationReservationView {
        code: item.code.clone(),
        label: text::truncate(&item.label, 40),
        window_days_text: item.window_days.to_string(),
        ttl_seconds_text: item.ttl_seconds.to_string(),
        unit: item.unit.clone(),
        budget: format!("{:.2}", item.budget),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_migration_reservation_views(items: &[MigrationReservation]) -> Vec<MigrationReservationView> {
    let mut views: Vec<MigrationReservationView> = Vec::new();
    for item in items.iter() {
        views.push(to_migration_reservation_view(item));
    }
    views
}

pub fn sum_migration_reservation_view(view: &MigrationReservationView) -> Result<f64, String> {
    let parsed_window_days = view
        .window_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    let parsed_ttl_seconds = view
        .ttl_seconds_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    Ok(parsed_window_days as f64 + (parsed_ttl_seconds as f64) + parsed_budget)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_migration_reservation("code-242", "label-506");
        let view = to_migration_reservation_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_migration_reservation_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
