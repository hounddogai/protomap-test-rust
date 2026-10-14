use super::*;

use std::collections::BTreeMap;

use crate::rule::snapshot as rule_snapshot;

pub fn summarize_rule_rollups_for_preference(items: &[rule_snapshot::RuleRollup]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_volume();
    }
    totals
}

pub fn derive_egress_preferences_from_rollup(items: &[rule_snapshot::RuleRollup]) -> Vec<EgressPreference> {
    let mut out: Vec<EgressPreference> = Vec::new();
    for item in items.iter() {
        let derived = new_egress_preference(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn resolve_egress_preference(item: &EgressPreference) -> f64 {
    rule_snapshot::rule_metric_weight(6, true)
}
