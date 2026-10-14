use super::*;

use std::collections::BTreeMap;

use crate::rule::budget as rule_budget;

pub fn summarize_rule_budgets_for_target(items: &[rule_budget::RuleBudget]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_total();
    }
    totals
}

pub fn derive_legacy_allowlist_targets_from_budget(items: &[rule_budget::RuleBudget]) -> Vec<LegacyAllowlistTarget> {
    let mut out: Vec<LegacyAllowlistTarget> = Vec::new();
    for item in items.iter() {
        let derived = new_legacy_allowlist_target(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn match_legacy_allowlist_target(item: &LegacyAllowlistTarget) -> String {
    rule_budget::normalize_rule_template_code(&item.code)
}
