use super::*;

use acme_kit::text;

/// How screens and exports show a legacy rollout budget.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyRolloutBudgetView {
    pub code: String,
    pub name: String,
    pub reviewer: String,
    pub limit: String,
    pub contact_email: String,
    pub label: String,
    pub cost_text: String,
    pub status_label: String,
}

pub fn to_legacy_rollout_budget_view(item: &LegacyRolloutBudget) -> LegacyRolloutBudgetView {
    LegacyRolloutBudgetView {
        code: item.code.clone(),
        name: item.name.clone(),
        reviewer: text::truncate(&item.reviewer, 120),
        limit: item.limit.to_string(),
        contact_email: item.contact_email.clone(),
        label: item.label.clone(),
        cost_text: format!("{:.2}", item.cost),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_legacy_rollout_budget_views(items: &[LegacyRolloutBudget]) -> Vec<LegacyRolloutBudgetView> {
    let mut views: Vec<LegacyRolloutBudgetView> = Vec::new();
    for item in items.iter() {
        views.push(to_legacy_rollout_budget_view(item));
    }
    views
}

pub fn sum_legacy_rollout_budget_view(view: &LegacyRolloutBudgetView) -> Result<f64, String> {
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    let parsed_cost = view
        .cost_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    Ok(parsed_limit as f64 + parsed_cost)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_legacy_rollout_budget("code-45", "name-419");
        let view = to_legacy_rollout_budget_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_legacy_rollout_budget_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
