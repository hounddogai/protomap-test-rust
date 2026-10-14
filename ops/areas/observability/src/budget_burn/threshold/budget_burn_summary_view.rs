use super::*;

use acme_kit::text;

/// How screens and exports show a budget burn summary.
#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnSummaryView {
    pub code: String,
    pub label: String,
    pub namespace_text: String,
    pub weight: String,
    pub locale_text: String,
    pub segment_text: String,
    pub owner_id_text: String,
    pub title: String,
    pub status_label: String,
}

pub fn to_budget_burn_summary_view(item: &BudgetBurnSummary) -> BudgetBurnSummaryView {
    BudgetBurnSummaryView {
        code: item.code.clone(),
        label: item.label.clone(),
        namespace_text: text::truncate(&item.namespace, 120),
        weight: format!("{:.2}", item.weight),
        locale_text: item.locale.clone(),
        segment_text: text::truncate(&item.segment, 80),
        owner_id_text: item.owner_id.clone(),
        title: text::truncate(&item.title, 64),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_budget_burn_summary_views(items: &[BudgetBurnSummary]) -> Vec<BudgetBurnSummaryView> {
    let mut views: Vec<BudgetBurnSummaryView> = Vec::new();
    for item in items.iter() {
        views.push(to_budget_burn_summary_view(item));
    }
    views
}

pub fn sum_budget_burn_summary_view(view: &BudgetBurnSummaryView) -> Result<f64, String> {
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    Ok(parsed_weight)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_budget_burn_summary("code-816", "label-336");
        let view = to_budget_burn_summary_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_budget_burn_summary_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
