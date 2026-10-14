use super::*;

use std::collections::BTreeMap;

use crate::retention::slot as retention_slot;

pub fn summarize_retention_slots_for_group(items: &[retention_slot::RetentionSlot]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_volume();
    }
    totals
}

pub fn derive_redaction_groups_from_slot(items: &[retention_slot::RetentionSlot]) -> Vec<RedactionGroup> {
    let mut out: Vec<RedactionGroup> = Vec::new();
    for item in items.iter() {
        let derived = new_redaction_group(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn compute_redaction_group(item: &RedactionGroup) -> f64 {
    retention_slot::retention_window_weight(4, true)
}
