use super::*;

use std::collections::BTreeMap;

use crate::overage::manifest as overage_manifest;

pub fn summarize_manual_overage_rules_for_window(
    items: &[overage_manifest::ManualOverageRule],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_total();
    }
    totals
}

pub fn derive_seasonal_grant_windows_from_rule(
    items: &[overage_manifest::ManualOverageRule],
) -> Vec<SeasonalGrantWindow> {
    let mut out: Vec<SeasonalGrantWindow> = Vec::new();
    for item in items.iter() {
        let derived = new_seasonal_grant_window(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn plan_seasonal_grant_window(item: &SeasonalGrantWindow) -> String {
    overage_manifest::tiered_overage_threshold_bucket(item.adjusted_total())
}
