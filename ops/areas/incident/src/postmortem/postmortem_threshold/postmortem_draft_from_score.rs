use super::*;

use std::collections::BTreeMap;

use crate::postmortem::override_item as postmortem_override_item;

pub fn summarize_rolling_postmortem_scores_for_draft(
    items: &[postmortem_override_item::RollingPostmortemScore],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_value();
    }
    totals
}

pub fn derive_postmortem_drafts_from_score(
    items: &[postmortem_override_item::RollingPostmortemScore],
) -> Vec<PostmortemDraft> {
    let mut out: Vec<PostmortemDraft> = Vec::new();
    for item in items.iter() {
        let derived = new_postmortem_draft(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn compute_postmortem_draft(item: &PostmortemDraft) -> String {
    postmortem_override_item::classify_postmortem_bundle(item.weighted_total())
}
