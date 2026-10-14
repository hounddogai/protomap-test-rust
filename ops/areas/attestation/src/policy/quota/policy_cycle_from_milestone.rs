use super::*;

use std::collections::BTreeMap;

use crate::nonce::baseline as nonce_baseline;

pub fn summarize_nonce_milestones_for_cycle(items: &[nonce_baseline::NonceMilestone]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_load();
    }
    totals
}

pub fn derive_policy_cycles_from_milestone(items: &[nonce_baseline::NonceMilestone]) -> Vec<PolicyCycle> {
    let mut out: Vec<PolicyCycle> = Vec::new();
    for item in items.iter() {
        let derived = new_policy_cycle(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn settle_policy_cycle(item: &PolicyCycle) -> String {
    nonce_baseline::normalize_nonce_slot_code(&item.code)
}
