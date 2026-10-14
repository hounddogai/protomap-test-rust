use super::*;

use std::collections::BTreeMap;

use crate::approval::agreement as approval_agreement;

pub fn summarize_approval_snapshots_for_template(
    items: &[approval_agreement::ApprovalSnapshot],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_load();
    }
    totals
}

pub fn derive_tiered_rollout_templates_from_snapshot(
    items: &[approval_agreement::ApprovalSnapshot],
) -> Vec<TieredRolloutTemplate> {
    let mut out: Vec<TieredRolloutTemplate> = Vec::new();
    for item in items.iter() {
        let derived = new_tiered_rollout_template(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn allocate_tiered_rollout_template(item: &TieredRolloutTemplate) -> String {
    approval_agreement::classify_adaptive_approval_agreement(item.effective_value())
}
