use super::*;

use std::collections::BTreeMap;

use crate::rule::entry as rule_entry;

pub fn summarize_rule_items_for_record(items: &[rule_entry::RuleItem]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_rule_records_from_item(items: &[rule_entry::RuleItem]) -> Vec<RuleRecord> {
    let mut out: Vec<RuleRecord> = Vec::new();
    for item in items.iter() {
        let derived = new_rule_record(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn score_rule_record(item: &RuleRecord) -> String {
    rule_entry::rule_entry_bucket(item.measured_value())
}
