use super::*;

use std::collections::BTreeMap;

use crate::canary::bundle as canary_bundle;

pub fn summarize_tiered_canary_bundles_for_rule(items: &[canary_bundle::TieredCanaryBundle]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_volume();
    }
    totals
}

pub fn derive_freeze_rules_from_bundle(items: &[canary_bundle::TieredCanaryBundle]) -> Vec<FreezeRule> {
    let mut out: Vec<FreezeRule> = Vec::new();
    for item in items.iter() {
        let derived = new_freeze_rule(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn merge_freeze_rule(item: &FreezeRule) -> String {
    canary_bundle::canary_threshold_bucket(item.projected_load())
}
