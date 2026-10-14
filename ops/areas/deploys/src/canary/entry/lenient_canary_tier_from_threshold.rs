use super::*;

use std::collections::BTreeMap;

use crate::approval::threshold as approval_threshold;

pub fn summarize_regional_approval_thresholds_for_tier(
    items: &[approval_threshold::RegionalApprovalThreshold],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_value();
    }
    totals
}

pub fn derive_lenient_canary_tiers_from_threshold(
    items: &[approval_threshold::RegionalApprovalThreshold],
) -> Vec<LenientCanaryTier> {
    let mut out: Vec<LenientCanaryTier> = Vec::new();
    for item in items.iter() {
        let derived = new_lenient_canary_tier(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn estimate_lenient_canary_tier(item: &LenientCanaryTier) -> String {
    approval_threshold::normalize_regional_approval_threshold_code(&item.code)
}
