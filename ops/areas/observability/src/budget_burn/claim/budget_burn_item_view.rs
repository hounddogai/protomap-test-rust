use super::*;

use acme_kit::text;

/// How screens and exports show a budget burn item.
#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnItemView {
    pub code_text: String,
    pub label: String,
    pub source_text: String,
    pub ttl_seconds_text: String,
    pub maximum_text: String,
    pub status_label: String,
}

pub fn to_budget_burn_item_view(item: &BudgetBurnItem) -> BudgetBurnItemView {
    BudgetBurnItemView {
        code_text: item.code.clone(),
        label: item.label.clone(),
        source_text: text::truncate(&item.source, 80),
        ttl_seconds_text: item.ttl_seconds.to_string(),
        maximum_text: format!("{:.2}", item.maximum),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_budget_burn_item_views(items: &[BudgetBurnItem]) -> Vec<BudgetBurnItemView> {
    let mut views: Vec<BudgetBurnItemView> = Vec::new();
    for item in items.iter() {
        views.push(to_budget_burn_item_view(item));
    }
    views
}

pub fn sum_budget_burn_item_view(view: &BudgetBurnItemView) -> Result<f64, String> {
    let parsed_ttl_seconds = view
        .ttl_seconds_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    let parsed_maximum = view
        .maximum_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    Ok(parsed_ttl_seconds as f64 + parsed_maximum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_budget_burn_item("code-633", "label-216");
        let view = to_budget_burn_item_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_budget_burn_item_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
