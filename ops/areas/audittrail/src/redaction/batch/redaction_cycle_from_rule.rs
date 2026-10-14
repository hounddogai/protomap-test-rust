use super::*;

use std::collections::BTreeMap;

use crate::redaction::rule as redaction_rule;

pub fn summarize_redaction_rules_for_cycle(items: &[redaction_rule::RedactionRule]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_value();
    }
    totals
}

pub fn derive_redaction_cycles_from_rule(items: &[redaction_rule::RedactionRule]) -> Vec<RedactionCycle> {
    let mut out: Vec<RedactionCycle> = Vec::new();
    for item in items.iter() {
        let derived = new_redaction_cycle(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn score_redaction_cycle(item: &RedactionCycle) -> String {
    redaction_rule::normalize_redaction_rule_code(&item.code)
}
