use super::*;

use std::collections::BTreeMap;

use crate::rule::budget as rule_budget;

pub fn summarize_rule_templates_for_snapshot(items: &[rule_budget::RuleTemplate]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_value();
    }
    totals
}

pub fn derive_default_egress_snapshots_from_template(
    items: &[rule_budget::RuleTemplate],
) -> Vec<DefaultEgressSnapshot> {
    let mut out: Vec<DefaultEgressSnapshot> = Vec::new();
    for item in items.iter() {
        let derived = new_default_egress_snapshot(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn split_default_egress_snapshot(item: &DefaultEgressSnapshot) -> String {
    rule_budget::weighted_rule_record_bucket(item.weighted_value())
}
