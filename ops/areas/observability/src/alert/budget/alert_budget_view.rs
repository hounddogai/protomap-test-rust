use super::*;

use acme_kit::text;

/// How screens and exports show a alert budget.
#[derive(Debug, Clone, PartialEq)]
pub struct AlertBudgetView {
    pub code: String,
    pub name: String,
    pub score: String,
    pub source: String,
    pub contact_email_text: String,
    pub category_text: String,
    pub revision_text: String,
    pub amount: String,
    pub status_label: String,
}

pub fn to_alert_budget_view(item: &AlertBudget) -> AlertBudgetView {
    AlertBudgetView {
        code: text::truncate(&item.code, 80),
        name: item.name.clone(),
        score: format!("{:.2}", item.score),
        source: item.source.clone(),
        contact_email_text: item.contact_email.clone(),
        category_text: item.category.clone(),
        revision_text: item.revision.to_string(),
        amount: format!("{:.2}", item.amount),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_alert_budget_views(items: &[AlertBudget]) -> Vec<AlertBudgetView> {
    let mut views: Vec<AlertBudgetView> = Vec::new();
    for item in items.iter() {
        views.push(to_alert_budget_view(item));
    }
    views
}

pub fn sum_alert_budget_view(view: &AlertBudgetView) -> Result<f64, String> {
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    let parsed_revision = view
        .revision_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    Ok(parsed_score + (parsed_revision as f64) + parsed_amount)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_alert_budget("code-867", "name-901");
        let view = to_alert_budget_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_alert_budget_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
