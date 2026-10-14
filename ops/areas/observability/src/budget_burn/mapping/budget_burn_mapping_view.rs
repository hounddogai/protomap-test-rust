use super::*;

use acme_kit::text;

/// How screens and exports show a budget burn mapping.
#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnMappingView {
    pub code: String,
    pub label: String,
    pub budget: String,
    pub capacity: String,
    pub limit: String,
    pub quota_units: String,
    pub status_label: String,
}

pub fn to_budget_burn_mapping_view(item: &BudgetBurnMapping) -> BudgetBurnMappingView {
    BudgetBurnMappingView {
        code: item.code.clone(),
        label: text::truncate(&item.label, 120),
        budget: format!("{:.2}", item.budget),
        capacity: item.capacity.to_string(),
        limit: item.limit.to_string(),
        quota_units: item.quota_units.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_budget_burn_mapping_views(items: &[BudgetBurnMapping]) -> Vec<BudgetBurnMappingView> {
    let mut views: Vec<BudgetBurnMappingView> = Vec::new();
    for item in items.iter() {
        views.push(to_budget_burn_mapping_view(item));
    }
    views
}

pub fn sum_budget_burn_mapping_view(view: &BudgetBurnMappingView) -> Result<f64, String> {
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    let parsed_capacity = view
        .capacity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    Ok(parsed_budget + (parsed_capacity as f64) + (parsed_limit as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_budget_burn_mapping("code-844", "label-601");
        let view = to_budget_burn_mapping_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_budget_burn_mapping_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
