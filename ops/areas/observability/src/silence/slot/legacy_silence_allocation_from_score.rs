use super::*;

use std::collections::BTreeMap;

use crate::silence::score as silence_score;

pub fn summarize_rolling_silence_scores_for_allocation(
    items: &[silence_score::RollingSilenceScore],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_volume();
    }
    totals
}

pub fn derive_legacy_silence_allocations_from_score(
    items: &[silence_score::RollingSilenceScore],
) -> Vec<LegacySilenceAllocation> {
    let mut out: Vec<LegacySilenceAllocation> = Vec::new();
    for item in items.iter() {
        let derived = new_legacy_silence_allocation(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn assign_legacy_silence_allocation(item: &LegacySilenceAllocation) -> String {
    silence_score::rolling_silence_score_bucket(item.projected_volume())
}
