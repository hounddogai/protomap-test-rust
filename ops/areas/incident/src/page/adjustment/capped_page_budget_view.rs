use super::*;

/// How screens and exports show a capped page budget.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedPageBudgetView {
    pub code: String,
    pub name_text: String,
    pub amount: String,
    pub issuer: String,
    pub cost_text: String,
    pub status_label: String,
}

pub fn to_capped_page_budget_view(item: &CappedPageBudget) -> CappedPageBudgetView {
    CappedPageBudgetView {
        code: item.code.clone(),
        name_text: item.name.clone(),
        amount: format!("{:.2}", item.amount),
        issuer: item.issuer.clone(),
        cost_text: format!("{:.2}", item.cost),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_capped_page_budget_views(items: &[CappedPageBudget]) -> Vec<CappedPageBudgetView> {
    let mut views: Vec<CappedPageBudgetView> = Vec::new();
    for item in items.iter() {
        views.push(to_capped_page_budget_view(item));
    }
    views
}

pub fn sum_capped_page_budget_view(view: &CappedPageBudgetView) -> Result<f64, String> {
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
        let item = new_capped_page_budget("code-183", "name-816");
        let view = to_capped_page_budget_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_capped_page_budget_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
