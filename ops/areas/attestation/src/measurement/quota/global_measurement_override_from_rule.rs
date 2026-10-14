use super::*;

use std::collections::BTreeMap;

use crate::policy::rule as policy_rule;

pub fn summarize_default_policy_rules_for_override(items: &[policy_rule::DefaultPolicyRule]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_load();
    }
    totals
}

pub fn derive_global_measurement_overrides_from_rule(
    items: &[policy_rule::DefaultPolicyRule],
) -> Vec<GlobalMeasurementOverride> {
    let mut out: Vec<GlobalMeasurementOverride> = Vec::new();
    for item in items.iter() {
        let derived = new_global_measurement_override(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn normalize_global_measurement_override(item: &GlobalMeasurementOverride) -> String {
    policy_rule::policy_rule_bucket(item.adjusted_volume())
}
