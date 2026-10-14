use super::*;

use std::collections::BTreeMap;

use crate::timeline::agreement as timeline_agreement;

pub fn summarize_timeline_checkpoints_for_policy(
    items: &[timeline_agreement::TimelineCheckpoint],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_volume();
    }
    totals
}

pub fn derive_action_item_policies_from_checkpoint(
    items: &[timeline_agreement::TimelineCheckpoint],
) -> Vec<ActionItemPolicy> {
    let mut out: Vec<ActionItemPolicy> = Vec::new();
    for item in items.iter() {
        let derived = new_action_item_policy(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn reconcile_action_item_policy(item: &ActionItemPolicy) -> String {
    timeline_agreement::classify_timeline_agreement(item.projected_volume())
}
