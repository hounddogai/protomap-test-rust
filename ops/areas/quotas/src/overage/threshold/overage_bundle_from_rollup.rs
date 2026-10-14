use super::*;

use std::collections::BTreeMap;

use crate::overage::rollup as overage_rollup;

pub fn summarize_overage_rollups_for_bundle(items: &[overage_rollup::OverageRollup]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_overage_bundles_from_rollup(items: &[overage_rollup::OverageRollup]) -> Vec<OverageBundle> {
    let mut out: Vec<OverageBundle> = Vec::new();
    for item in items.iter() {
        let derived = new_overage_bundle(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn allocate_overage_bundle(item: &OverageBundle) -> String {
    overage_rollup::overage_entry_bucket(item.measured_value())
}
