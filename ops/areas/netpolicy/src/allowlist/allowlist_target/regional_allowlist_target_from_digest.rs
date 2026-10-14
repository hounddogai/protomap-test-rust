use super::*;

use std::collections::BTreeMap;

use crate::egress::digest as egress_digest;

pub fn summarize_egress_digests_for_target(items: &[egress_digest::EgressDigest]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_regional_allowlist_targets_from_digest(
    items: &[egress_digest::EgressDigest],
) -> Vec<RegionalAllowlistTarget> {
    let mut out: Vec<RegionalAllowlistTarget> = Vec::new();
    for item in items.iter() {
        let derived = new_regional_allowlist_target(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn settle_regional_allowlist_target(item: &RegionalAllowlistTarget) -> String {
    egress_digest::classify_egress_digest(item.projected_exposure())
}
