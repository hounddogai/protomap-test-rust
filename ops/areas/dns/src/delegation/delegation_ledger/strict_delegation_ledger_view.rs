use super::*;

use acme_kit::text;

/// How screens and exports show a strict delegation ledger.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictDelegationLedgerView {
    pub code: String,
    pub name_text: String,
    pub threshold_text: String,
    pub key_id: String,
    pub cluster_text: String,
    pub status_label: String,
}

pub fn to_strict_delegation_ledger_view(item: &StrictDelegationLedger) -> StrictDelegationLedgerView {
    StrictDelegationLedgerView {
        code: text::truncate(&item.code, 40),
        name_text: item.name.clone(),
        threshold_text: format!("{:.2}", item.threshold),
        key_id: item.key_id.clone(),
        cluster_text: text::truncate(&item.cluster, 40),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_strict_delegation_ledger_views(items: &[StrictDelegationLedger]) -> Vec<StrictDelegationLedgerView> {
    let mut views: Vec<StrictDelegationLedgerView> = Vec::new();
    for item in items.iter() {
        views.push(to_strict_delegation_ledger_view(item));
    }
    views
}

pub fn sum_strict_delegation_ledger_view(view: &StrictDelegationLedgerView) -> Result<f64, String> {
    let parsed_threshold = view
        .threshold_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    Ok(parsed_threshold)
}
