use super::*;

use std::collections::BTreeMap;

use crate::retention::bundle as retention_bundle;

pub fn summarize_tiered_retention_baselines_for_agreement(
    items: &[retention_bundle::TieredRetentionBaseline],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_volume();
    }
    totals
}

pub fn derive_audit_event_agreements_from_baseline(
    items: &[retention_bundle::TieredRetentionBaseline],
) -> Vec<AuditEventAgreement> {
    let mut out: Vec<AuditEventAgreement> = Vec::new();
    for item in items.iter() {
        let derived = new_audit_event_agreement(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn reconcile_audit_event_agreement(item: &AuditEventAgreement) -> String {
    retention_bundle::normalize_retention_bundle_code(&item.code)
}
