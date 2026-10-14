use super::*;

use acme_kit::text;

/// How screens and exports show a default quote budget.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultQuoteBudgetView {
    pub code: String,
    pub title_text: String,
    pub cluster_text: String,
    pub floor_value_text: String,
    pub external_ref_text: String,
    pub status_label: String,
}

pub fn to_default_quote_budget_view(item: &DefaultQuoteBudget) -> DefaultQuoteBudgetView {
    DefaultQuoteBudgetView {
        code: item.code.clone(),
        title_text: text::truncate(&item.title, 40),
        cluster_text: text::truncate(&item.cluster, 80),
        floor_value_text: format!("{:.2}", item.floor_value),
        external_ref_text: item.external_ref.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_default_quote_budget_views(items: &[DefaultQuoteBudget]) -> Vec<DefaultQuoteBudgetView> {
    let mut views: Vec<DefaultQuoteBudgetView> = Vec::new();
    for item in items.iter() {
        views.push(to_default_quote_budget_view(item));
    }
    views
}

pub fn sum_default_quote_budget_view(view: &DefaultQuoteBudgetView) -> Result<f64, String> {
    let parsed_floor_value = view
        .floor_value_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    Ok(parsed_floor_value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_default_quote_budget("code-505", "title-969");
        let view = to_default_quote_budget_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_default_quote_budget_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
