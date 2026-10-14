use super::*;

use acme_kit::text;

/// How screens and exports show a budget burn allocation.
#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnAllocationView {
    pub code_text: String,
    pub name: String,
    pub score: String,
    pub active: String,
    pub cluster: String,
    pub grace_days: String,
    pub threshold: String,
    pub approved: String,
    pub weight: String,
    pub margin: String,
    pub owner_id_text: String,
    pub status_label: String,
}

pub fn to_budget_burn_allocation_view(item: &BudgetBurnAllocation) -> BudgetBurnAllocationView {
    BudgetBurnAllocationView {
        code_text: text::truncate(&item.code, 40),
        name: item.name.clone(),
        score: format!("{:.2}", item.score),
        active: format!("{}", item.active),
        cluster: item.cluster.clone(),
        grace_days: item.grace_days.to_string(),
        threshold: format!("{:.2}", item.threshold),
        approved: format!("{}", item.approved),
        weight: format!("{:.2}", item.weight),
        margin: format!("{:.2}", item.margin),
        owner_id_text: text::truncate(&item.owner_id, 64),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_budget_burn_allocation_views(items: &[BudgetBurnAllocation]) -> Vec<BudgetBurnAllocationView> {
    let mut views: Vec<BudgetBurnAllocationView> = Vec::new();
    for item in items.iter() {
        views.push(to_budget_burn_allocation_view(item));
    }
    views
}

pub fn sum_budget_burn_allocation_view(view: &BudgetBurnAllocationView) -> Result<f64, String> {
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    let parsed_threshold = view
        .threshold
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    Ok(parsed_score + (parsed_grace_days as f64) + parsed_threshold)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_budget_burn_allocation("code-879", "name-157");
        let view = to_budget_burn_allocation_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_budget_burn_allocation_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
