use super::*;

use std::collections::BTreeMap;

use crate::isolation::baseline as isolation_baseline;

pub fn summarize_isolation_rules_for_notice(items: &[isolation_baseline::IsolationRule]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_isolation_notices_from_rule(items: &[isolation_baseline::IsolationRule]) -> Vec<IsolationNotice> {
    let mut out: Vec<IsolationNotice> = Vec::new();
    for item in items.iter() {
        let derived = new_isolation_notice(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn summarize_isolation_notice(item: &IsolationNotice) -> f64 {
    isolation_baseline::isolation_baseline_weight(5, true)
}
