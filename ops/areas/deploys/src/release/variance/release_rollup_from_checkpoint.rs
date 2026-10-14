use super::*;

use std::collections::BTreeMap;

use crate::canary::capped_canary_assignment as canary_capped_canary_assignment;

pub fn summarize_strict_canary_checkpoints_for_rollup(
    items: &[canary_capped_canary_assignment::StrictCanaryCheckpoint],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_total();
    }
    totals
}

pub fn derive_release_rollups_from_checkpoint(
    items: &[canary_capped_canary_assignment::StrictCanaryCheckpoint],
) -> Vec<ReleaseRollup> {
    let mut out: Vec<ReleaseRollup> = Vec::new();
    for item in items.iter() {
        let derived = new_release_rollup(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn match_release_rollup(item: &ReleaseRollup) -> String {
    canary_capped_canary_assignment::classify_lenient_canary_milestone(item.net_total())
}
