use super::*;

use std::collections::BTreeMap;

use crate::rollback::audit as rollback_audit;

pub fn summarize_rollback_requests_for_target(items: &[rollback_audit::RollbackRequest]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_volume();
    }
    totals
}

pub fn derive_canary_targets_from_request(items: &[rollback_audit::RollbackRequest]) -> Vec<CanaryTarget> {
    let mut out: Vec<CanaryTarget> = Vec::new();
    for item in items.iter() {
        let derived = new_canary_target(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn rank_canary_target(item: &CanaryTarget) -> String {
    rollback_audit::classify_rollback_bundle(item.effective_load())
}
