use super::*;

use std::collections::BTreeMap;

use crate::redaction::redaction_group as redaction_redaction_group;

pub fn summarize_global_redaction_groups_for_target(
    items: &[redaction_redaction_group::GlobalRedactionGroup],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_total();
    }
    totals
}

pub fn derive_manual_sink_targets_from_group(
    items: &[redaction_redaction_group::GlobalRedactionGroup],
) -> Vec<ManualSinkTarget> {
    let mut out: Vec<ManualSinkTarget> = Vec::new();
    for item in items.iter() {
        let derived = new_manual_sink_target(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn assign_manual_sink_target(item: &ManualSinkTarget) -> f64 {
    redaction_redaction_group::redaction_milestone_weight(7, true)
}
