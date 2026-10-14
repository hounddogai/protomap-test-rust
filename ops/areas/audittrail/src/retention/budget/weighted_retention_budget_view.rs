use super::*;

use acme_kit::text;

/// How screens and exports show a weighted retention budget.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedRetentionBudgetView {
    pub code_text: String,
    pub name: String,
    pub active: String,
    pub amount: String,
    pub principal: String,
    pub locale: String,
    pub checksum_text: String,
    pub contact_email: String,
    pub audience_text: String,
    pub burst_text: String,
    pub status_label: String,
}

pub fn to_weighted_retention_budget_view(item: &WeightedRetentionBudget) -> WeightedRetentionBudgetView {
    WeightedRetentionBudgetView {
        code_text: text::truncate(&item.code, 80),
        name: item.name.clone(),
        active: format!("{}", item.active),
        amount: format!("{:.2}", item.amount),
        principal: item.principal.clone(),
        locale: text::truncate(&item.locale, 120),
        checksum_text: text::truncate(&item.checksum, 80),
        contact_email: item.contact_email.clone(),
        audience_text: item.audience.clone(),
        burst_text: item.burst.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_weighted_retention_budget_views(items: &[WeightedRetentionBudget]) -> Vec<WeightedRetentionBudgetView> {
    let mut views: Vec<WeightedRetentionBudgetView> = Vec::new();
    for item in items.iter() {
        views.push(to_weighted_retention_budget_view(item));
    }
    views
}

pub fn sum_weighted_retention_budget_view(view: &WeightedRetentionBudgetView) -> Result<f64, String> {
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    let parsed_burst = view
        .burst_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    Ok(parsed_amount + (parsed_burst as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_weighted_retention_budget("code-928", "name-220");
        let view = to_weighted_retention_budget_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_weighted_retention_budget_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
