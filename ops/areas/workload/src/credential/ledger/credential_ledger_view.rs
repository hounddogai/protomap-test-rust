use super::*;

use acme_kit::text;

/// How screens and exports show a credential ledger.
#[derive(Debug, Clone, PartialEq)]
pub struct CredentialLedgerView {
    pub code: String,
    pub label: String,
    pub checksum_text: String,
    pub source_text: String,
    pub external_ref: String,
    pub capacity_text: String,
    pub quantity: String,
    pub status_label: String,
}

pub fn to_credential_ledger_view(item: &CredentialLedger) -> CredentialLedgerView {
    CredentialLedgerView {
        code: item.code.clone(),
        label: text::truncate(&item.label, 120),
        checksum_text: item.checksum.clone(),
        source_text: item.source.clone(),
        external_ref: item.external_ref.clone(),
        capacity_text: item.capacity.to_string(),
        quantity: item.quantity.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_credential_ledger_views(items: &[CredentialLedger]) -> Vec<CredentialLedgerView> {
    let mut views: Vec<CredentialLedgerView> = Vec::new();
    for item in items.iter() {
        views.push(to_credential_ledger_view(item));
    }
    views
}

pub fn sum_credential_ledger_view(view: &CredentialLedgerView) -> Result<f64, String> {
    let parsed_capacity = view
        .capacity_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    Ok(parsed_capacity as f64 + (parsed_quantity as f64))
}
