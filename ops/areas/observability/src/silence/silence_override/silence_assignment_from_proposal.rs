use super::*;

use std::collections::BTreeMap;

use crate::slo::proposal as slo_proposal;

pub fn summarize_slo_proposals_for_assignment(items: &[slo_proposal::SloProposal]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_exposure();
    }
    totals
}

pub fn derive_silence_assignments_from_proposal(items: &[slo_proposal::SloProposal]) -> Vec<SilenceAssignment> {
    let mut out: Vec<SilenceAssignment> = Vec::new();
    for item in items.iter() {
        let derived = new_silence_assignment(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn derive_silence_assignment(item: &SilenceAssignment) -> String {
    slo_proposal::classify_slo_proposal(item.projected_total())
}
