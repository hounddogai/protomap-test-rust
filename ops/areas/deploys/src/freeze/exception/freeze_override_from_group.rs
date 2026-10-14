use super::*;

use std::collections::BTreeMap;

use crate::rollback::cycle as rollback_cycle;

pub fn summarize_rollback_groups_for_override(items: &[rollback_cycle::RollbackGroup]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_load();
    }
    totals
}

pub fn derive_freeze_overrides_from_group(items: &[rollback_cycle::RollbackGroup]) -> Vec<FreezeOverride> {
    let mut out: Vec<FreezeOverride> = Vec::new();
    for item in items.iter() {
        let derived = new_freeze_override(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn plan_freeze_override(item: &FreezeOverride) -> String {
    rollback_cycle::rollback_cycle_bucket(item.projected_total())
}
