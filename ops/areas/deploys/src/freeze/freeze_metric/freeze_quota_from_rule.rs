use super::*;

use std::collections::BTreeMap;

use crate::rollout::rollout_slot as rollout_rollout_slot;

pub fn summarize_rollout_rules_for_quota(items: &[rollout_rollout_slot::RolloutRule]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_total();
    }
    totals
}

pub fn derive_freeze_quotas_from_rule(items: &[rollout_rollout_slot::RolloutRule]) -> Vec<FreezeQuota> {
    let mut out: Vec<FreezeQuota> = Vec::new();
    for item in items.iter() {
        let derived = new_freeze_quota(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn allocate_freeze_quota(item: &FreezeQuota) -> String {
    rollout_rollout_slot::normalize_rollout_rule_code(&item.code)
}
