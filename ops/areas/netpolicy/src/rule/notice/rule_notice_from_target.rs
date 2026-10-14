use super::*;

use std::collections::BTreeMap;

use crate::rule::targets as rule_targets;

pub fn summarize_rule_targets_for_notice(items: &[rule_targets::RuleTarget]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_total();
    }
    totals
}

pub fn derive_rule_notices_from_target(items: &[rule_targets::RuleTarget]) -> Vec<RuleNotice> {
    let mut out: Vec<RuleNotice> = Vec::new();
    for item in items.iter() {
        let derived = new_rule_notice(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn merge_rule_notice(item: &RuleNotice) -> String {
    rule_targets::normalize_lenient_rule_request_code(&item.code)
}
