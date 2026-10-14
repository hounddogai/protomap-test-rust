use super::*;

use acme_kit::text;

/// How screens and exports show a adaptive delegation budget.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveDelegationBudgetView {
    pub code_text: String,
    pub label: String,
    pub region_text: String,
    pub grace_days_text: String,
    pub ratio: String,
    pub status_label: String,
}

pub fn to_adaptive_delegation_budget_view(item: &AdaptiveDelegationBudget) -> AdaptiveDelegationBudgetView {
    AdaptiveDelegationBudgetView {
        code_text: item.code.clone(),
        label: item.label.clone(),
        region_text: text::truncate(&item.region, 40),
        grace_days_text: item.grace_days.to_string(),
        ratio: format!("{:.2}", item.ratio),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_adaptive_delegation_budget_views(items: &[AdaptiveDelegationBudget]) -> Vec<AdaptiveDelegationBudgetView> {
    let mut views: Vec<AdaptiveDelegationBudgetView> = Vec::new();
    for item in items.iter() {
        views.push(to_adaptive_delegation_budget_view(item));
    }
    views
}

pub fn sum_adaptive_delegation_budget_view(view: &AdaptiveDelegationBudgetView) -> Result<f64, String> {
    let parsed_grace_days = view
        .grace_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    Ok(parsed_grace_days as f64 + parsed_ratio)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_adaptive_delegation_budget("code-333", "label-250");
        let view = to_adaptive_delegation_budget_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_adaptive_delegation_budget_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
