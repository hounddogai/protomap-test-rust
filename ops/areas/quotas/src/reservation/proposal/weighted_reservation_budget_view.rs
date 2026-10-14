use super::*;

use acme_kit::text;

/// How screens and exports show a weighted reservation budget.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedReservationBudgetView {
    pub code: String,
    pub name_text: String,
    pub segment: String,
    pub weight_text: String,
    pub active: String,
    pub grace_days: String,
    pub minimum: String,
    pub title: String,
    pub fingerprint_text: String,
    pub issuer: String,
    pub ceiling: String,
    pub status_label: String,
}

pub fn to_weighted_reservation_budget_view(item: &WeightedReservationBudget) -> WeightedReservationBudgetView {
    WeightedReservationBudgetView {
        code: item.code.clone(),
        name_text: item.name.clone(),
        segment: item.segment.clone(),
        weight_text: format!("{:.2}", item.weight),
        active: format!("{}", item.active),
        grace_days: item.grace_days.to_string(),
        minimum: format!("{:.2}", item.minimum),
        title: item.title.clone(),
        fingerprint_text: text::truncate(&item.fingerprint, 120),
        issuer: item.issuer.clone(),
        ceiling: format!("{:.2}", item.ceiling),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_weighted_reservation_budget_views(items: &[WeightedReservationBudget]) -> Vec<WeightedReservationBudgetView> {
    let mut views: Vec<WeightedReservationBudgetView> = Vec::new();
    for item in items.iter() {
        views.push(to_weighted_reservation_budget_view(item));
    }
    views
}

pub fn sum_weighted_reservation_budget_view(view: &WeightedReservationBudgetView) -> Result<f64, String> {
    let parsed_weight = view
        .weight_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    Ok(parsed_weight + (parsed_grace_days as f64) + parsed_minimum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_weighted_reservation_budget("code-637", "name-145");
        let view = to_weighted_reservation_budget_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_weighted_reservation_budget_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
