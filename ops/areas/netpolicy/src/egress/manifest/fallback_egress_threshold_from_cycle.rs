use super::*;

use std::collections::BTreeMap;

use crate::rule::policy as rule_policy;

pub fn summarize_seasonal_rule_cycles_for_threshold(items: &[rule_policy::SeasonalRuleCycle]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_value();
    }
    totals
}

pub fn derive_fallback_egress_thresholds_from_cycle(
    items: &[rule_policy::SeasonalRuleCycle],
) -> Vec<FallbackEgressThreshold> {
    let mut out: Vec<FallbackEgressThreshold> = Vec::new();
    for item in items.iter() {
        let derived = new_fallback_egress_threshold(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn derive_fallback_egress_threshold(item: &FallbackEgressThreshold) -> String {
    rule_policy::normalize_rule_summary_code(&item.code)
}
