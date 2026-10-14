use super::*;

use std::collections::BTreeMap;

use crate::page::ledger as page_ledger;

pub fn summarize_legacy_page_plans_for_rule(items: &[page_ledger::LegacyPagePlan]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_value();
    }
    totals
}

pub fn derive_postmortem_rules_from_plan(items: &[page_ledger::LegacyPagePlan]) -> Vec<PostmortemRule> {
    let mut out: Vec<PostmortemRule> = Vec::new();
    for item in items.iter() {
        let derived = new_postmortem_rule(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn match_postmortem_rule(item: &PostmortemRule) -> String {
    page_ledger::normalize_global_page_variance_code(&item.code)
}
