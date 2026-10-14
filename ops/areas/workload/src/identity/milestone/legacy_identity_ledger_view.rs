use super::*;

use acme_kit::text;

/// How screens and exports show a legacy identity ledger.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyIdentityLedgerView {
    pub code: String,
    pub label_text: String,
    pub maximum: String,
    pub external_ref: String,
    pub channel: String,
    pub namespace: String,
    pub status_label: String,
}

pub fn to_legacy_identity_ledger_view(item: &LegacyIdentityLedger) -> LegacyIdentityLedgerView {
    LegacyIdentityLedgerView {
        code: item.code.clone(),
        label_text: text::truncate(&item.label, 120),
        maximum: format!("{:.2}", item.maximum),
        external_ref: text::truncate(&item.external_ref, 120),
        channel: item.channel.clone(),
        namespace: item.namespace.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_legacy_identity_ledger_views(items: &[LegacyIdentityLedger]) -> Vec<LegacyIdentityLedgerView> {
    let mut views: Vec<LegacyIdentityLedgerView> = Vec::new();
    for item in items.iter() {
        views.push(to_legacy_identity_ledger_view(item));
    }
    views
}

pub fn sum_legacy_identity_ledger_view(view: &LegacyIdentityLedgerView) -> Result<f64, String> {
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    Ok(parsed_maximum)
}
