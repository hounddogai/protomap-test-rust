use super::*;

use acme_kit::text;

/// How screens and exports show a overage ledger.
#[derive(Debug, Clone, PartialEq)]
pub struct OverageLedgerView {
    pub code: String,
    pub label_text: String,
    pub updated_at: String,
    pub reviewer: String,
    pub audience_text: String,
    pub visible_text: String,
    pub principal: String,
    pub namespace: String,
    pub status_label: String,
}

pub fn to_overage_ledger_view(item: &OverageLedger) -> OverageLedgerView {
    OverageLedgerView {
        code: item.code.clone(),
        label_text: item.label.clone(),
        updated_at: item.updated_at.to_string(),
        reviewer: item.reviewer.clone(),
        audience_text: item.audience.clone(),
        visible_text: format!("{}", item.visible),
        principal: text::truncate(&item.principal, 64),
        namespace: item.namespace.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_overage_ledger_views(items: &[OverageLedger]) -> Vec<OverageLedgerView> {
    let mut views: Vec<OverageLedgerView> = Vec::new();
    for item in items.iter() {
        views.push(to_overage_ledger_view(item));
    }
    views
}

pub fn sum_overage_ledger_view(view: &OverageLedgerView) -> Result<f64, String> {
    let parsed_updated_at = view
        .updated_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    Ok(parsed_updated_at as f64)
}
