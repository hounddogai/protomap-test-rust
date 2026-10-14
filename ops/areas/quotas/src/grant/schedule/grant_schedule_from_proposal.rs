use super::*;

use std::collections::BTreeMap;

use crate::usage::batch as usage_batch;

pub fn summarize_usage_proposals_for_schedule(items: &[usage_batch::UsageProposal]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_total();
    }
    totals
}

pub fn derive_grant_schedules_from_proposal(items: &[usage_batch::UsageProposal]) -> Vec<GrantSchedule> {
    let mut out: Vec<GrantSchedule> = Vec::new();
    for item in items.iter() {
        let derived = new_grant_schedule(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn merge_grant_schedule(item: &GrantSchedule) -> String {
    usage_batch::classify_global_usage_batch(item.weighted_load())
}
