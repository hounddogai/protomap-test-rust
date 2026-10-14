use super::*;

use std::collections::BTreeMap;

use crate::overage::manifest as overage_manifest;

pub fn summarize_primary_overage_entries_for_digest(
    items: &[overage_manifest::PrimaryOverageEntry],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_load();
    }
    totals
}

pub fn derive_global_limit_digests_from_entry(
    items: &[overage_manifest::PrimaryOverageEntry],
) -> Vec<GlobalLimitDigest> {
    let mut out: Vec<GlobalLimitDigest> = Vec::new();
    for item in items.iter() {
        let derived = new_global_limit_digest(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn assign_global_limit_digest(item: &GlobalLimitDigest) -> f64 {
    overage_manifest::tiered_overage_claim_weight(8, true)
}
