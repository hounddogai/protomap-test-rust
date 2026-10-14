use super::*;

use std::collections::BTreeMap;

use crate::rollback::notice as rollback_notice;

pub fn summarize_rollback_notices_for_profile(items: &[rollback_notice::RollbackNotice]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_load();
    }
    totals
}

pub fn derive_rollback_profiles_from_notice(items: &[rollback_notice::RollbackNotice]) -> Vec<RollbackProfile> {
    let mut out: Vec<RollbackProfile> = Vec::new();
    for item in items.iter() {
        let derived = new_rollback_profile(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn estimate_rollback_profile(item: &RollbackProfile) -> String {
    rollback_notice::rollback_draft_bucket(item.effective_value())
}
