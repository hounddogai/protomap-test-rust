use super::*;

use std::collections::BTreeMap;

use crate::release::summary as release_summary;

pub fn summarize_release_scores_for_plan(items: &[release_summary::ReleaseScore]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_exposure();
    }
    totals
}

pub fn derive_fallback_rollout_plans_from_score(items: &[release_summary::ReleaseScore]) -> Vec<FallbackRolloutPlan> {
    let mut out: Vec<FallbackRolloutPlan> = Vec::new();
    for item in items.iter() {
        let derived = new_fallback_rollout_plan(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn score_fallback_rollout_plan(item: &FallbackRolloutPlan) -> String {
    release_summary::normalize_pending_release_proposal_code(&item.code)
}
