use super::*;

use acme_kit::text;

/// How screens and exports show a batched delegation budget.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedDelegationBudgetView {
    pub code: String,
    pub name_text: String,
    pub cipher: String,
    pub category: String,
    pub discount_text: String,
    pub reviewer: String,
    pub status_label: String,
}

pub fn to_batched_delegation_budget_view(item: &BatchedDelegationBudget) -> BatchedDelegationBudgetView {
    BatchedDelegationBudgetView {
        code: item.code.clone(),
        name_text: item.name.clone(),
        cipher: text::truncate(&item.cipher, 40),
        category: text::truncate(&item.category, 120),
        discount_text: format!("{:.2}", item.discount),
        reviewer: item.reviewer.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_batched_delegation_budget_views(items: &[BatchedDelegationBudget]) -> Vec<BatchedDelegationBudgetView> {
    let mut views: Vec<BatchedDelegationBudgetView> = Vec::new();
    for item in items.iter() {
        views.push(to_batched_delegation_budget_view(item));
    }
    views
}

pub fn sum_batched_delegation_budget_view(view: &BatchedDelegationBudgetView) -> Result<f64, String> {
    let parsed_discount = view
        .discount_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    Ok(parsed_discount)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_batched_delegation_budget("code-210", "name-607");
        let view = to_batched_delegation_budget_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_batched_delegation_budget_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
