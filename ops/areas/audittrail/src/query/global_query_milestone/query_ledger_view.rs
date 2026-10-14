use super::*;

/// How screens and exports show a query ledger.
#[derive(Debug, Clone, PartialEq)]
pub struct QueryLedgerView {
    pub code: String,
    pub label: String,
    pub retries: String,
    pub notes: String,
    pub unit: String,
    pub ends_at_text: String,
    pub owner_id: String,
    pub contact_email: String,
    pub status_label: String,
}

pub fn to_query_ledger_view(item: &QueryLedger) -> QueryLedgerView {
    QueryLedgerView {
        code: item.code.clone(),
        label: item.label.clone(),
        retries: item.retries.to_string(),
        notes: item.notes.clone(),
        unit: item.unit.clone(),
        ends_at_text: item.ends_at.to_string(),
        owner_id: item.owner_id.clone(),
        contact_email: item.contact_email.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_query_ledger_views(items: &[QueryLedger]) -> Vec<QueryLedgerView> {
    let mut views: Vec<QueryLedgerView> = Vec::new();
    for item in items.iter() {
        views.push(to_query_ledger_view(item));
    }
    views
}

pub fn sum_query_ledger_view(view: &QueryLedgerView) -> Result<f64, String> {
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    let parsed_ends_at = view
        .ends_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    Ok(parsed_retries as f64 + (parsed_ends_at as f64))
}
