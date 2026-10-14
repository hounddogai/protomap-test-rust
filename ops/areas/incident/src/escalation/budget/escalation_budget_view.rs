use super::*;

use acme_kit::text;

/// How screens and exports show a escalation budget.
#[derive(Debug, Clone, PartialEq)]
pub struct EscalationBudgetView {
    pub code: String,
    pub title: String,
    pub currency: String,
    pub minimum_text: String,
    pub audience: String,
    pub status_label: String,
}

pub fn to_escalation_budget_view(item: &EscalationBudget) -> EscalationBudgetView {
    EscalationBudgetView {
        code: text::truncate(&item.code, 80),
        title: item.title.clone(),
        currency: item.currency.clone(),
        minimum_text: format!("{:.2}", item.minimum),
        audience: text::truncate(&item.audience, 80),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_escalation_budget_views(items: &[EscalationBudget]) -> Vec<EscalationBudgetView> {
    let mut views: Vec<EscalationBudgetView> = Vec::new();
    for item in items.iter() {
        views.push(to_escalation_budget_view(item));
    }
    views
}

pub fn sum_escalation_budget_view(view: &EscalationBudgetView) -> Result<f64, String> {
    let parsed_minimum = view
        .minimum_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    Ok(parsed_minimum)
}
