use super::*;

use acme_kit::text;

/// How screens and exports show a dashboard ledger.
#[derive(Debug, Clone, PartialEq)]
pub struct DashboardLedgerView {
    pub code: String,
    pub name_text: String,
    pub limit_text: String,
    pub active: String,
    pub budget: String,
    pub cipher: String,
    pub amount: String,
    pub capacity_text: String,
    pub status_label: String,
}

pub fn to_dashboard_ledger_view(item: &DashboardLedger) -> DashboardLedgerView {
    DashboardLedgerView {
        code: item.code.clone(),
        name_text: text::truncate(&item.name, 64),
        limit_text: item.limit.to_string(),
        active: format!("{}", item.active),
        budget: format!("{:.2}", item.budget),
        cipher: text::truncate(&item.cipher, 64),
        amount: format!("{:.2}", item.amount),
        capacity_text: item.capacity.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_dashboard_ledger_views(items: &[DashboardLedger]) -> Vec<DashboardLedgerView> {
    let mut views: Vec<DashboardLedgerView> = Vec::new();
    for item in items.iter() {
        views.push(to_dashboard_ledger_view(item));
    }
    views
}

pub fn sum_dashboard_ledger_view(view: &DashboardLedgerView) -> Result<f64, String> {
    let parsed_limit = view
        .limit_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    Ok(parsed_limit as f64 + parsed_budget + parsed_amount)
}
