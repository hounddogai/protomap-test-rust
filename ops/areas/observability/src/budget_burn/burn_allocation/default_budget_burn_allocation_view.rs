use super::*;

use acme_kit::text;

/// How screens and exports show a default budget burn allocation.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultBudgetBurnAllocationView {
    pub code: String,
    pub title_text: String,
    pub cost: String,
    pub score: String,
    pub cipher_text: String,
    pub status_label: String,
}

pub fn to_default_budget_burn_allocation_view(item: &DefaultBudgetBurnAllocation) -> DefaultBudgetBurnAllocationView {
    DefaultBudgetBurnAllocationView {
        code: text::truncate(&item.code, 64),
        title_text: item.title.clone(),
        cost: format!("{:.2}", item.cost),
        score: format!("{:.2}", item.score),
        cipher_text: text::truncate(&item.cipher, 64),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_default_budget_burn_allocation_views(
    items: &[DefaultBudgetBurnAllocation],
) -> Vec<DefaultBudgetBurnAllocationView> {
    let mut views: Vec<DefaultBudgetBurnAllocationView> = Vec::new();
    for item in items.iter() {
        views.push(to_default_budget_burn_allocation_view(item));
    }
    views
}

pub fn sum_default_budget_burn_allocation_view(view: &DefaultBudgetBurnAllocationView) -> Result<f64, String> {
    let parsed_cost = view
        .cost
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    Ok(parsed_cost + parsed_score)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_default_budget_burn_allocation("code-517", "title-144");
        let view = to_default_budget_burn_allocation_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_default_budget_burn_allocation_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
