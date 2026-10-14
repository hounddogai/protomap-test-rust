use super::*;

use std::collections::BTreeMap;

use crate::release::tier as release_tier;

pub fn summarize_adaptive_release_tiers_for_target(
    items: &[release_tier::AdaptiveReleaseTier],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_volume();
    }
    totals
}

pub fn derive_approval_targets_from_tier(items: &[release_tier::AdaptiveReleaseTier]) -> Vec<ApprovalTarget> {
    let mut out: Vec<ApprovalTarget> = Vec::new();
    for item in items.iter() {
        let derived = new_approval_target(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn project_approval_target(item: &ApprovalTarget) -> String {
    release_tier::classify_adaptive_release_tier(item.weighted_exposure())
}
