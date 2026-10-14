use super::*;

use acme_kit::text;

/// How screens and exports show a egress ledger.
#[derive(Debug, Clone, PartialEq)]
pub struct EgressLedgerView {
    pub code: String,
    pub label: String,
    pub limit: String,
    pub owner_id: String,
    pub amount: String,
    pub status_label: String,
}

pub fn to_egress_ledger_view(item: &EgressLedger) -> EgressLedgerView {
    EgressLedgerView {
        code: text::truncate(&item.code, 64),
        label: item.label.clone(),
        limit: item.limit.to_string(),
        owner_id: item.owner_id.clone(),
        amount: format!("{:.2}", item.amount),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_egress_ledger_views(items: &[EgressLedger]) -> Vec<EgressLedgerView> {
    let mut views: Vec<EgressLedgerView> = Vec::new();
    for item in items.iter() {
        views.push(to_egress_ledger_view(item));
    }
    views
}

pub fn sum_egress_ledger_view(view: &EgressLedgerView) -> Result<f64, String> {
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    Ok(parsed_limit as f64 + parsed_amount)
}
