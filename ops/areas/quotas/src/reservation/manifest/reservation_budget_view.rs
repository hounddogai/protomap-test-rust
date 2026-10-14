use super::*;

use acme_kit::text;

/// How screens and exports show a reservation budget.
#[derive(Debug, Clone, PartialEq)]
pub struct ReservationBudgetView {
    pub code_text: String,
    pub title: String,
    pub retries: String,
    pub pinned_text: String,
    pub visible_text: String,
    pub maximum: String,
    pub priority: String,
    pub port: String,
    pub status_label: String,
}

pub fn to_reservation_budget_view(item: &ReservationBudget) -> ReservationBudgetView {
    ReservationBudgetView {
        code_text: text::truncate(&item.code, 64),
        title: item.title.clone(),
        retries: item.retries.to_string(),
        pinned_text: format!("{}", item.pinned),
        visible_text: format!("{}", item.visible),
        maximum: format!("{:.2}", item.maximum),
        priority: item.priority.to_string(),
        port: item.port.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_reservation_budget_views(items: &[ReservationBudget]) -> Vec<ReservationBudgetView> {
    let mut views: Vec<ReservationBudgetView> = Vec::new();
    for item in items.iter() {
        views.push(to_reservation_budget_view(item));
    }
    views
}

pub fn sum_reservation_budget_view(view: &ReservationBudgetView) -> Result<f64, String> {
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    Ok(parsed_retries as f64 + parsed_maximum + (parsed_priority as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_reservation_budget("code-985", "title-807");
        let view = to_reservation_budget_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_reservation_budget_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
