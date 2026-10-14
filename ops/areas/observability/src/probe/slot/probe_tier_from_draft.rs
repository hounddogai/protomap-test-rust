use super::*;

use std::collections::BTreeMap;

use crate::slo::proposal as slo_proposal;

pub fn summarize_slo_drafts_for_tier(items: &[slo_proposal::SloDraft]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_volume();
    }
    totals
}

pub fn derive_probe_tiers_from_draft(items: &[slo_proposal::SloDraft]) -> Vec<ProbeTier> {
    let mut out: Vec<ProbeTier> = Vec::new();
    for item in items.iter() {
        let derived = new_probe_tier(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn match_probe_tier(item: &ProbeTier) -> String {
    slo_proposal::classify_slo_proposal(item.projected_total())
}
