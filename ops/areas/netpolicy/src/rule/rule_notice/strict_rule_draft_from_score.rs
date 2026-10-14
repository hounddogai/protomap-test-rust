use super::*;

use std::collections::BTreeMap;

use crate::peering::score as peering_score;

pub fn summarize_peering_scores_for_draft(items: &[peering_score::PeeringScore]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_exposure();
    }
    totals
}

pub fn derive_strict_rule_drafts_from_score(items: &[peering_score::PeeringScore]) -> Vec<StrictRuleDraft> {
    let mut out: Vec<StrictRuleDraft> = Vec::new();
    for item in items.iter() {
        let derived = new_strict_rule_draft(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn project_strict_rule_draft(item: &StrictRuleDraft) -> f64 {
    peering_score::peering_score_weight(6, true)
}
