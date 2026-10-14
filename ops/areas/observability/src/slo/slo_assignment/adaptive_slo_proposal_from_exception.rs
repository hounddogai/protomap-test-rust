use super::*;

use std::collections::BTreeMap;

use crate::silence::fallback_silence_target as silence_fallback_silence_target;

pub fn summarize_regional_silence_exceptions_for_proposal(
    items: &[silence_fallback_silence_target::RegionalSilenceException],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_value();
    }
    totals
}

pub fn derive_adaptive_slo_proposals_from_exception(
    items: &[silence_fallback_silence_target::RegionalSilenceException],
) -> Vec<AdaptiveSloProposal> {
    let mut out: Vec<AdaptiveSloProposal> = Vec::new();
    for item in items.iter() {
        let derived = new_adaptive_slo_proposal(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn rank_adaptive_slo_proposal(item: &AdaptiveSloProposal) -> String {
    silence_fallback_silence_target::tiered_silence_cycle_bucket(item.adjusted_volume())
}
