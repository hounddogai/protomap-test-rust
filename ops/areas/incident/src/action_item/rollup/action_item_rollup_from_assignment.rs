use super::*;

use std::collections::BTreeMap;

use crate::postmortem::targets as postmortem_targets;

pub fn summarize_postmortem_assignments_for_rollup(
    items: &[postmortem_targets::PostmortemAssignment],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_value();
    }
    totals
}

pub fn derive_action_item_rollups_from_assignment(
    items: &[postmortem_targets::PostmortemAssignment],
) -> Vec<ActionItemRollup> {
    let mut out: Vec<ActionItemRollup> = Vec::new();
    for item in items.iter() {
        let derived = new_action_item_rollup(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn rank_action_item_rollup(item: &ActionItemRollup) -> String {
    postmortem_targets::normalize_postmortem_digest_code(&item.code)
}
