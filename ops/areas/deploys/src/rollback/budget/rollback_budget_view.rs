use super::*;

use acme_kit::text;

/// How screens and exports show a rollback budget.
#[derive(Debug, Clone, PartialEq)]
pub struct RollbackBudgetView {
    pub code: String,
    pub label: String,
    pub limit: String,
    pub score: String,
    pub burst_text: String,
    pub fingerprint: String,
    pub tenant_id_text: String,
    pub port_text: String,
    pub notes: String,
    pub status_label: String,
}

pub fn to_rollback_budget_view(item: &RollbackBudget) -> RollbackBudgetView {
    RollbackBudgetView {
        code: item.code.clone(),
        label: item.label.clone(),
        limit: item.limit.to_string(),
        score: format!("{:.2}", item.score),
        burst_text: item.burst.to_string(),
        fingerprint: item.fingerprint.clone(),
        tenant_id_text: item.tenant_id.clone(),
        port_text: item.port.to_string(),
        notes: text::truncate(&item.notes, 120),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rollback_budget_views(items: &[RollbackBudget]) -> Vec<RollbackBudgetView> {
    let mut views: Vec<RollbackBudgetView> = Vec::new();
    for item in items.iter() {
        views.push(to_rollback_budget_view(item));
    }
    views
}

pub fn sum_rollback_budget_view(view: &RollbackBudgetView) -> Result<f64, String> {
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    let parsed_burst = view
        .burst_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    Ok(parsed_limit as f64 + parsed_score + (parsed_burst as f64))
}
