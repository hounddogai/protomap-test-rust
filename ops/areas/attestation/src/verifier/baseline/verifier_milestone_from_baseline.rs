use super::*;

use std::collections::BTreeMap;

use crate::verifier::segment as verifier_segment;

pub fn summarize_verifier_baselines_for_milestone(
    items: &[verifier_segment::VerifierBaseline],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_volume();
    }
    totals
}

pub fn derive_verifier_milestones_from_baseline(
    items: &[verifier_segment::VerifierBaseline],
) -> Vec<VerifierMilestone> {
    let mut out: Vec<VerifierMilestone> = Vec::new();
    for item in items.iter() {
        let derived = new_verifier_milestone(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn summarize_verifier_milestone(item: &VerifierMilestone) -> f64 {
    verifier_segment::regional_verifier_segment_weight(2, true)
}
