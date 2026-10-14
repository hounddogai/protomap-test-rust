use super::*;

use std::collections::BTreeMap;

use crate::release::ledger as release_ledger;

pub fn summarize_release_segments_for_request(items: &[release_ledger::ReleaseSegment]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_volume();
    }
    totals
}

pub fn derive_rollout_requests_from_segment(items: &[release_ledger::ReleaseSegment]) -> Vec<RolloutRequest> {
    let mut out: Vec<RolloutRequest> = Vec::new();
    for item in items.iter() {
        let derived = new_rollout_request(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn estimate_rollout_request(item: &RolloutRequest) -> String {
    release_ledger::classify_release_plan(item.weighted_load())
}
