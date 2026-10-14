use super::*;

use std::collections::BTreeMap;

use crate::isolation::entry as isolation_entry;

pub fn summarize_adaptive_isolation_templates_for_target(
    items: &[isolation_entry::AdaptiveIsolationTemplate],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_value();
    }
    totals
}

pub fn derive_suspension_targets_from_template(
    items: &[isolation_entry::AdaptiveIsolationTemplate],
) -> Vec<SuspensionTarget> {
    let mut out: Vec<SuspensionTarget> = Vec::new();
    for item in items.iter() {
        let derived = new_suspension_target(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn reconcile_suspension_target(item: &SuspensionTarget) -> String {
    isolation_entry::classify_isolation_audit(item.effective_load())
}
