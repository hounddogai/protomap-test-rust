use super::*;

use std::collections::BTreeMap;

use crate::silence::allocation as silence_allocation;

pub fn summarize_silence_policies_for_cycle(items: &[silence_allocation::SilencePolicy]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_exposure();
    }
    totals
}

pub fn derive_batched_slo_cycles_from_policy(items: &[silence_allocation::SilencePolicy]) -> Vec<BatchedSloCycle> {
    let mut out: Vec<BatchedSloCycle> = Vec::new();
    for item in items.iter() {
        let derived = new_batched_slo_cycle(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn plan_batched_slo_cycle(item: &BatchedSloCycle) -> String {
    silence_allocation::global_silence_allocation_bucket(item.measured_value())
}
