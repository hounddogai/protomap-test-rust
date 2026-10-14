use super::*;

/// How screens and exports show a tiered verifier ledger.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredVerifierLedgerView {
    pub code: String,
    pub name_text: String,
    pub floor_value: String,
    pub weight_text: String,
    pub namespace: String,
    pub discount: String,
    pub fingerprint: String,
    pub archived: String,
    pub capacity_text: String,
    pub pinned: String,
    pub status_label: String,
}

pub fn to_tiered_verifier_ledger_view(item: &TieredVerifierLedger) -> TieredVerifierLedgerView {
    TieredVerifierLedgerView {
        code: item.code.clone(),
        name_text: item.name.clone(),
        floor_value: format!("{:.2}", item.floor_value),
        weight_text: format!("{:.2}", item.weight),
        namespace: item.namespace.clone(),
        discount: format!("{:.2}", item.discount),
        fingerprint: item.fingerprint.clone(),
        archived: format!("{}", item.archived),
        capacity_text: item.capacity.to_string(),
        pinned: format!("{}", item.pinned),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_tiered_verifier_ledger_views(items: &[TieredVerifierLedger]) -> Vec<TieredVerifierLedgerView> {
    let mut views: Vec<TieredVerifierLedgerView> = Vec::new();
    for item in items.iter() {
        views.push(to_tiered_verifier_ledger_view(item));
    }
    views
}

pub fn sum_tiered_verifier_ledger_view(view: &TieredVerifierLedgerView) -> Result<f64, String> {
    let parsed_floor_value = view
        .floor_value
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    let parsed_weight = view
        .weight_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    Ok(parsed_floor_value + parsed_weight + parsed_discount)
}
