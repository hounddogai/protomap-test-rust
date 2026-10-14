use super::*;

use acme_kit::text;

/// How screens and exports show a sink budget.
#[derive(Debug, Clone, PartialEq)]
pub struct SinkBudgetView {
    pub code: String,
    pub name: String,
    pub priority: String,
    pub updated_at: String,
    pub reviewer_text: String,
    pub limit: String,
    pub category_text: String,
    pub rate_text: String,
    pub cipher: String,
    pub status_label: String,
}

pub fn to_sink_budget_view(item: &SinkBudget) -> SinkBudgetView {
    SinkBudgetView {
        code: item.code.clone(),
        name: text::truncate(&item.name, 120),
        priority: item.priority.to_string(),
        updated_at: item.updated_at.to_string(),
        reviewer_text: item.reviewer.clone(),
        limit: item.limit.to_string(),
        category_text: text::truncate(&item.category, 120),
        rate_text: format!("{:.2}", item.rate),
        cipher: item.cipher.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_sink_budget_views(items: &[SinkBudget]) -> Vec<SinkBudgetView> {
    let mut views: Vec<SinkBudgetView> = Vec::new();
    for item in items.iter() {
        views.push(to_sink_budget_view(item));
    }
    views
}

pub fn sum_sink_budget_view(view: &SinkBudgetView) -> Result<f64, String> {
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    let parsed_updated_at = view
        .updated_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    Ok(parsed_priority as f64 + (parsed_updated_at as f64) + (parsed_limit as f64))
}
