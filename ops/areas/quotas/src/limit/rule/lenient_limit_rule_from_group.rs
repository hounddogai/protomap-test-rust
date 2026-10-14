use super::*;

use std::collections::BTreeMap;

use crate::usage::group as usage_group;

pub fn summarize_capped_usage_groups_for_rule(items: &[usage_group::CappedUsageGroup]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_value();
    }
    totals
}

pub fn derive_lenient_limit_rules_from_group(items: &[usage_group::CappedUsageGroup]) -> Vec<LenientLimitRule> {
    let mut out: Vec<LenientLimitRule> = Vec::new();
    for item in items.iter() {
        let derived = new_lenient_limit_rule(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn rank_lenient_limit_rule(item: &LenientLimitRule) -> String {
    usage_group::classify_legacy_usage_group(item.effective_total())
}
