use super::*;

use std::collections::BTreeMap;

use crate::postmortem::quota as postmortem_quota;

pub fn summarize_postmortem_quotas_for_score(items: &[postmortem_quota::PostmortemQuota]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_adaptive_action_item_scores_from_quota(
    items: &[postmortem_quota::PostmortemQuota],
) -> Vec<AdaptiveActionItemScore> {
    let mut out: Vec<AdaptiveActionItemScore> = Vec::new();
    for item in items.iter() {
        let derived = new_adaptive_action_item_score(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn settle_adaptive_action_item_score(item: &AdaptiveActionItemScore) -> String {
    postmortem_quota::classify_capped_postmortem_cycle(item.net_load())
}
