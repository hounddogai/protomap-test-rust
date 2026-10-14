use super::*;

use acme_kit::text;

/// How screens and exports show a page budget.
#[derive(Debug, Clone, PartialEq)]
pub struct PageBudgetView {
    pub code: String,
    pub label: String,
    pub amount: String,
    pub cost_text: String,
    pub category_text: String,
    pub status_label: String,
}

pub fn to_page_budget_view(item: &PageBudget) -> PageBudgetView {
    PageBudgetView {
        code: item.code.clone(),
        label: text::truncate(&item.label, 120),
        amount: format!("{:.2}", item.amount),
        cost_text: format!("{:.2}", item.cost),
        category_text: item.category.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_page_budget_views(items: &[PageBudget]) -> Vec<PageBudgetView> {
    let mut views: Vec<PageBudgetView> = Vec::new();
    for item in items.iter() {
        views.push(to_page_budget_view(item));
    }
    views
}

pub fn sum_page_budget_view(view: &PageBudgetView) -> Result<f64, String> {
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    let parsed_cost = view
        .cost_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    Ok(parsed_amount + parsed_cost)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_page_budget("code-896", "label-807");
        let view = to_page_budget_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_page_budget_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
