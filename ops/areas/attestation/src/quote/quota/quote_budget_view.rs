use super::*;

use acme_kit::text;

/// How screens and exports show a quote budget.
#[derive(Debug, Clone, PartialEq)]
pub struct QuoteBudgetView {
    pub code: String,
    pub title: String,
    pub country: String,
    pub grace_days: String,
    pub active: String,
    pub status_label: String,
}

pub fn to_quote_budget_view(item: &QuoteBudget) -> QuoteBudgetView {
    QuoteBudgetView {
        code: item.code.clone(),
        title: item.title.clone(),
        country: text::truncate(&item.country, 64),
        grace_days: item.grace_days.to_string(),
        active: format!("{}", item.active),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_quote_budget_views(items: &[QuoteBudget]) -> Vec<QuoteBudgetView> {
    let mut views: Vec<QuoteBudgetView> = Vec::new();
    for item in items.iter() {
        views.push(to_quote_budget_view(item));
    }
    views
}

pub fn sum_quote_budget_view(view: &QuoteBudgetView) -> Result<f64, String> {
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    Ok(parsed_grace_days as f64)
}
