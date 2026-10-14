use super::*;

use std::collections::BTreeMap;

use crate::rollout::request as rollout_request;

pub fn summarize_rollout_requests_for_preference(items: &[rollout_request::RolloutRequest]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_load();
    }
    totals
}

pub fn derive_rollback_preferences_from_request(items: &[rollout_request::RolloutRequest]) -> Vec<RollbackPreference> {
    let mut out: Vec<RollbackPreference> = Vec::new();
    for item in items.iter() {
        let derived = new_rollback_preference(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn classify_rollback_preference_extra(item: &RollbackPreference) -> String {
    rollout_request::normalize_rollout_request_code(&item.code)
}
