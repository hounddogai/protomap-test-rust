use super::*;

use std::collections::BTreeMap;

use crate::allowlist::digest as allowlist_digest;

pub fn summarize_allowlist_digests_for_bundle(items: &[allowlist_digest::AllowlistDigest]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_exposure();
    }
    totals
}

pub fn derive_flow_log_bundles_from_digest(items: &[allowlist_digest::AllowlistDigest]) -> Vec<FlowLogBundle> {
    let mut out: Vec<FlowLogBundle> = Vec::new();
    for item in items.iter() {
        let derived = new_flow_log_bundle(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn classify_flow_log_bundle_extra(item: &FlowLogBundle) -> f64 {
    allowlist_digest::allowlist_digest_weight(5, true)
}
