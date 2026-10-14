use super::*;

use std::collections::BTreeMap;

use crate::isolation::review as isolation_review;

pub fn summarize_isolation_summaries_for_rule(items: &[isolation_review::IsolationSummary]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_total();
    }
    totals
}

pub fn derive_isolation_rules_from_summary(items: &[isolation_review::IsolationSummary]) -> Vec<IsolationRule> {
    let mut out: Vec<IsolationRule> = Vec::new();
    for item in items.iter() {
        let derived = new_isolation_rule(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn normalize_isolation_rule(item: &IsolationRule) -> String {
    isolation_review::normalize_isolation_review_code(&item.code)
}
