use super::*;

use acme_kit::text;

/// How screens and exports show a manual budget burn reservation.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualBudgetBurnReservationView {
    pub code: String,
    pub name: String,
    pub scope: String,
    pub contact_email: String,
    pub minimum: String,
    pub country: String,
    pub status_label: String,
}

pub fn to_manual_budget_burn_reservation_view(item: &ManualBudgetBurnReservation) -> ManualBudgetBurnReservationView {
    ManualBudgetBurnReservationView {
        code: item.code.clone(),
        name: item.name.clone(),
        scope: item.scope.clone(),
        contact_email: text::truncate(&item.contact_email, 64),
        minimum: format!("{:.2}", item.minimum),
        country: text::truncate(&item.country, 40),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_manual_budget_burn_reservation_views(
    items: &[ManualBudgetBurnReservation],
) -> Vec<ManualBudgetBurnReservationView> {
    let mut views: Vec<ManualBudgetBurnReservationView> = Vec::new();
    for item in items.iter() {
        views.push(to_manual_budget_burn_reservation_view(item));
    }
    views
}

pub fn sum_manual_budget_burn_reservation_view(view: &ManualBudgetBurnReservationView) -> Result<f64, String> {
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
        let item = new_manual_budget_burn_reservation("code-512", "name-376");
        let view = to_manual_budget_burn_reservation_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_manual_budget_burn_reservation_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
