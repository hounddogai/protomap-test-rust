use super::*;

use acme_kit::text;

/// How screens and exports show a budget burn ledger.
#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnLedgerView {
    pub code: String,
    pub label_text: String,
    pub archived: String,
    pub limit: String,
    pub name: String,
    pub locale: String,
    pub ceiling_text: String,
    pub starts_at_text: String,
    pub ttl_seconds: String,
    pub status_label: String,
}

pub fn to_budget_burn_ledger_view(item: &BudgetBurnLedger) -> BudgetBurnLedgerView {
    BudgetBurnLedgerView {
        code: item.code.clone(),
        label_text: item.label.clone(),
        archived: format!("{}", item.archived),
        limit: item.limit.to_string(),
        name: item.name.clone(),
        locale: text::truncate(&item.locale, 64),
        ceiling_text: format!("{:.2}", item.ceiling),
        starts_at_text: item.starts_at.to_string(),
        ttl_seconds: item.ttl_seconds.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_budget_burn_ledger_views(items: &[BudgetBurnLedger]) -> Vec<BudgetBurnLedgerView> {
    let mut views: Vec<BudgetBurnLedgerView> = Vec::new();
    for item in items.iter() {
        views.push(to_budget_burn_ledger_view(item));
    }
    views
}

pub fn sum_budget_burn_ledger_view(view: &BudgetBurnLedgerView) -> Result<f64, String> {
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    let parsed_ceiling = view
        .ceiling_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    let parsed_starts_at = view
        .starts_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    Ok(parsed_limit as f64 + parsed_ceiling + (parsed_starts_at as f64))
}
