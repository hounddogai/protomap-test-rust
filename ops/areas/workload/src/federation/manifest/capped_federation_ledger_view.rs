use super::*;

/// How screens and exports show a capped federation ledger.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedFederationLedgerView {
    pub code: String,
    pub title: String,
    pub owner_id_text: String,
    pub principal: String,
    pub minimum: String,
    pub cipher: String,
    pub archived: String,
    pub external_ref: String,
    pub status_label: String,
}

pub fn to_capped_federation_ledger_view(item: &CappedFederationLedger) -> CappedFederationLedgerView {
    CappedFederationLedgerView {
        code: item.code.clone(),
        title: item.title.clone(),
        owner_id_text: item.owner_id.clone(),
        principal: item.principal.clone(),
        minimum: format!("{:.2}", item.minimum),
        cipher: item.cipher.clone(),
        archived: format!("{}", item.archived),
        external_ref: item.external_ref.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_capped_federation_ledger_views(items: &[CappedFederationLedger]) -> Vec<CappedFederationLedgerView> {
    let mut views: Vec<CappedFederationLedgerView> = Vec::new();
    for item in items.iter() {
        views.push(to_capped_federation_ledger_view(item));
    }
    views
}

pub fn sum_capped_federation_ledger_view(view: &CappedFederationLedgerView) -> Result<f64, String> {
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    Ok(parsed_minimum)
}
