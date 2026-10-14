use super::*;

use acme_kit::text;

/// How screens and exports show a budget burn estimate.
#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnEstimateView {
    pub code: String,
    pub title: String,
    pub ceiling: String,
    pub rate: String,
    pub name_text: String,
    pub retries: String,
    pub tenant_id: String,
    pub port: String,
    pub attempts_text: String,
    pub cost: String,
    pub limit: String,
    pub status_label: String,
}

pub fn to_budget_burn_estimate_view(item: &BudgetBurnEstimate) -> BudgetBurnEstimateView {
    BudgetBurnEstimateView {
        code: text::truncate(&item.code, 40),
        title: item.title.clone(),
        ceiling: format!("{:.2}", item.ceiling),
        rate: format!("{:.2}", item.rate),
        name_text: text::truncate(&item.name, 64),
        retries: item.retries.to_string(),
        tenant_id: text::truncate(&item.tenant_id, 120),
        port: item.port.to_string(),
        attempts_text: item.attempts.to_string(),
        cost: format!("{:.2}", item.cost),
        limit: item.limit.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_budget_burn_estimate_views(items: &[BudgetBurnEstimate]) -> Vec<BudgetBurnEstimateView> {
    let mut views: Vec<BudgetBurnEstimateView> = Vec::new();
    for item in items.iter() {
        views.push(to_budget_burn_estimate_view(item));
    }
    views
}

pub fn sum_budget_burn_estimate_view(view: &BudgetBurnEstimateView) -> Result<f64, String> {
    let parsed_ceiling = view
        .ceiling
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    Ok(parsed_ceiling + parsed_rate + (parsed_retries as f64))
}
