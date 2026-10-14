use super::*;

use std::collections::BTreeMap;

use crate::allowlist::snapshot as allowlist_snapshot;

pub fn summarize_allowlist_variances_for_agreement(
    items: &[allowlist_snapshot::AllowlistVariance],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_egress_agreements_from_variance(items: &[allowlist_snapshot::AllowlistVariance]) -> Vec<EgressAgreement> {
    let mut out: Vec<EgressAgreement> = Vec::new();
    for item in items.iter() {
        let derived = new_egress_agreement(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn apply_egress_agreement(item: &EgressAgreement) -> f64 {
    allowlist_snapshot::allowlist_quota_weight(9, true)
}
