use super::*;

use acme_kit::text;

/// How screens and exports show a isolation ledger.
#[derive(Debug, Clone, PartialEq)]
pub struct IsolationLedgerView {
    pub code_text: String,
    pub name: String,
    pub description_text: String,
    pub rate: String,
    pub weight: String,
    pub title_text: String,
    pub minimum: String,
    pub fingerprint: String,
    pub duration_minutes: String,
    pub status_label: String,
}

pub fn to_isolation_ledger_view(item: &IsolationLedger) -> IsolationLedgerView {
    IsolationLedgerView {
        code_text: item.code.clone(),
        name: item.name.clone(),
        description_text: text::truncate(&item.description, 120),
        rate: format!("{:.2}", item.rate),
        weight: format!("{:.2}", item.weight),
        title_text: item.title.clone(),
        minimum: format!("{:.2}", item.minimum),
        fingerprint: item.fingerprint.clone(),
        duration_minutes: item.duration_minutes.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_isolation_ledger_views(items: &[IsolationLedger]) -> Vec<IsolationLedgerView> {
    let mut views: Vec<IsolationLedgerView> = Vec::new();
    for item in items.iter() {
        views.push(to_isolation_ledger_view(item));
    }
    views
}

pub fn sum_isolation_ledger_view(view: &IsolationLedgerView) -> Result<f64, String> {
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    Ok(parsed_rate + parsed_weight + parsed_minimum)
}
