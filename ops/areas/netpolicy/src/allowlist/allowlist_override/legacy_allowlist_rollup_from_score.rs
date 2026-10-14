use super::*;

use std::collections::BTreeMap;

use crate::peering::score as peering_score;

pub fn summarize_peering_scores_for_rollup(items: &[peering_score::PeeringScore]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_exposure();
    }
    totals
}

pub fn derive_legacy_allowlist_rollups_from_score(items: &[peering_score::PeeringScore]) -> Vec<LegacyAllowlistRollup> {
    let mut out: Vec<LegacyAllowlistRollup> = Vec::new();
    for item in items.iter() {
        let derived = new_legacy_allowlist_rollup(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn match_legacy_allowlist_rollup(item: &LegacyAllowlistRollup) -> String {
    peering_score::classify_peering_score(item.effective_volume())
}
