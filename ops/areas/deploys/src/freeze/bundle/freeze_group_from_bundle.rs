use super::*;

use std::collections::BTreeMap;

use crate::rollback::audit as rollback_audit;

pub fn summarize_rollback_bundles_for_group(items: &[rollback_audit::RollbackBundle]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_load();
    }
    totals
}

pub fn derive_freeze_groups_from_bundle(items: &[rollback_audit::RollbackBundle]) -> Vec<FreezeGroup> {
    let mut out: Vec<FreezeGroup> = Vec::new();
    for item in items.iter() {
        let derived = new_freeze_group(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn allocate_freeze_group_extra(item: &FreezeGroup) -> String {
    rollback_audit::classify_rollback_bundle(item.effective_load())
}
