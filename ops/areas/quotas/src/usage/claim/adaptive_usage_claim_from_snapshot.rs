use super::*;

use std::collections::BTreeMap;

use crate::overage::overage_manifest as overage_overage_manifest;

pub fn summarize_regional_overage_snapshots_for_claim(
    items: &[overage_overage_manifest::RegionalOverageSnapshot],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_value();
    }
    totals
}

pub fn derive_adaptive_usage_claims_from_snapshot(
    items: &[overage_overage_manifest::RegionalOverageSnapshot],
) -> Vec<AdaptiveUsageClaim> {
    let mut out: Vec<AdaptiveUsageClaim> = Vec::new();
    for item in items.iter() {
        let derived = new_adaptive_usage_claim(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn merge_adaptive_usage_claim(item: &AdaptiveUsageClaim) -> String {
    overage_overage_manifest::normalize_fallback_overage_manifest_code(&item.code)
}
