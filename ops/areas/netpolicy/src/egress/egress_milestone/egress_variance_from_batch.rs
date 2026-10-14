use super::*;

use std::collections::BTreeMap;

use crate::rule::batch as rule_batch;

pub fn summarize_adaptive_rule_batches_for_variance(items: &[rule_batch::AdaptiveRuleBatch]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_volume();
    }
    totals
}

pub fn derive_egress_variances_from_batch(items: &[rule_batch::AdaptiveRuleBatch]) -> Vec<EgressVariance> {
    let mut out: Vec<EgressVariance> = Vec::new();
    for item in items.iter() {
        let derived = new_egress_variance(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn project_egress_variance(item: &EgressVariance) -> String {
    rule_batch::classify_adaptive_rule_batch(item.net_load())
}
