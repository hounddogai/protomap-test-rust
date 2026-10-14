use super::*;

use acme_kit::text;

/// How screens and exports show a action item budget.
#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemBudgetView {
    pub code: String,
    pub name: String,
    pub expires_at: String,
    pub pinned: String,
    pub quota_units: String,
    pub amount: String,
    pub updated_at_text: String,
    pub status_label: String,
}

pub fn to_action_item_budget_view(item: &ActionItemBudget) -> ActionItemBudgetView {
    ActionItemBudgetView {
        code: text::truncate(&item.code, 40),
        name: item.name.clone(),
        expires_at: item.expires_at.to_string(),
        pinned: format!("{}", item.pinned),
        quota_units: item.quota_units.to_string(),
        amount: format!("{:.2}", item.amount),
        updated_at_text: item.updated_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_action_item_budget_views(items: &[ActionItemBudget]) -> Vec<ActionItemBudgetView> {
    let mut views: Vec<ActionItemBudgetView> = Vec::new();
    for item in items.iter() {
        views.push(to_action_item_budget_view(item));
    }
    views
}

pub fn sum_action_item_budget_view(view: &ActionItemBudgetView) -> Result<f64, String> {
    let parsed_expires_at = view
        .expires_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    let parsed_quota_units = view
        .quota_units
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    Ok(parsed_expires_at as f64 + (parsed_quota_units as f64) + parsed_amount)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_action_item_budget("code-541", "name-925");
        let view = to_action_item_budget_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_action_item_budget_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
