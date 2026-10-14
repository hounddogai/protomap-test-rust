use super::*;

use acme_kit::text;

/// How screens and exports show a batched budget burn rollup.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedBudgetBurnRollupView {
    pub code: String,
    pub name: String,
    pub limit: String,
    pub archived_text: String,
    pub namespace: String,
    pub ratio: String,
    pub revision: String,
    pub status_label: String,
}

pub fn to_batched_budget_burn_rollup_view(item: &BatchedBudgetBurnRollup) -> BatchedBudgetBurnRollupView {
    BatchedBudgetBurnRollupView {
        code: item.code.clone(),
        name: item.name.clone(),
        limit: item.limit.to_string(),
        archived_text: format!("{}", item.archived),
        namespace: text::truncate(&item.namespace, 40),
        ratio: format!("{:.2}", item.ratio),
        revision: item.revision.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_batched_budget_burn_rollup_views(items: &[BatchedBudgetBurnRollup]) -> Vec<BatchedBudgetBurnRollupView> {
    let mut views: Vec<BatchedBudgetBurnRollupView> = Vec::new();
    for item in items.iter() {
        views.push(to_batched_budget_burn_rollup_view(item));
    }
    views
}

pub fn sum_batched_budget_burn_rollup_view(view: &BatchedBudgetBurnRollupView) -> Result<f64, String> {
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    let parsed_revision = view
        .revision
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    Ok(parsed_limit as f64 + parsed_ratio + (parsed_revision as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_batched_budget_burn_rollup("code-38", "name-711");
        let view = to_batched_budget_burn_rollup_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_batched_budget_burn_rollup_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
