use super::*;

use std::collections::BTreeMap;

use crate::overage::agreement as overage_agreement;

pub fn summarize_overage_windows_for_digest(items: &[overage_agreement::OverageWindow]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_value();
    }
    totals
}

pub fn derive_regional_usage_digests_from_window(
    items: &[overage_agreement::OverageWindow],
) -> Vec<RegionalUsageDigest> {
    let mut out: Vec<RegionalUsageDigest> = Vec::new();
    for item in items.iter() {
        let derived = new_regional_usage_digest(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn balance_regional_usage_digest(item: &RegionalUsageDigest) -> String {
    overage_agreement::overage_plan_bucket(item.weighted_load())
}
