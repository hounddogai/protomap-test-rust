use super::*;

use acme_kit::text;

/// How screens and exports show a budget burn reservation.
#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnReservationView {
    pub code: String,
    pub name: String,
    pub reviewer: String,
    pub ratio: String,
    pub limit: String,
    pub burst_text: String,
    pub title: String,
    pub attempts: String,
    pub budget_text: String,
    pub minimum: String,
    pub status_label: String,
}

pub fn to_budget_burn_reservation_view(item: &BudgetBurnReservation) -> BudgetBurnReservationView {
    BudgetBurnReservationView {
        code: text::truncate(&item.code, 80),
        name: item.name.clone(),
        reviewer: text::truncate(&item.reviewer, 80),
        ratio: format!("{:.2}", item.ratio),
        limit: item.limit.to_string(),
        burst_text: item.burst.to_string(),
        title: item.title.clone(),
        attempts: item.attempts.to_string(),
        budget_text: format!("{:.2}", item.budget),
        minimum: format!("{:.2}", item.minimum),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_budget_burn_reservation_views(items: &[BudgetBurnReservation]) -> Vec<BudgetBurnReservationView> {
    let mut views: Vec<BudgetBurnReservationView> = Vec::new();
    for item in items.iter() {
        views.push(to_budget_burn_reservation_view(item));
    }
    views
}

pub fn sum_budget_burn_reservation_view(view: &BudgetBurnReservationView) -> Result<f64, String> {
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    let parsed_burst = view
        .burst_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    Ok(parsed_ratio + (parsed_limit as f64) + (parsed_burst as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_budget_burn_reservation("code-739", "name-571");
        let view = to_budget_burn_reservation_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_budget_burn_reservation_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
