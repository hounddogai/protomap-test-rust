use super::*;

use std::collections::BTreeMap;

use crate::egress::plan as egress_plan;

pub fn summarize_egress_plans_for_checkpoint(items: &[egress_plan::EgressPlan]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_volume();
    }
    totals
}

pub fn derive_tiered_allowlist_checkpoints_from_plan(
    items: &[egress_plan::EgressPlan],
) -> Vec<TieredAllowlistCheckpoint> {
    let mut out: Vec<TieredAllowlistCheckpoint> = Vec::new();
    for item in items.iter() {
        let derived = new_tiered_allowlist_checkpoint(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn match_tiered_allowlist_checkpoint(item: &TieredAllowlistCheckpoint) -> String {
    egress_plan::normalize_egress_plan_code(&item.code)
}
