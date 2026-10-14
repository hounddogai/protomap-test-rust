use super::*;

use std::collections::BTreeMap;

use crate::action_item::score as action_item_score;

pub fn summarize_action_item_metrics_for_claim(items: &[action_item_score::ActionItemMetric]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_load();
    }
    totals
}

pub fn derive_global_postmortem_claims_from_metric(
    items: &[action_item_score::ActionItemMetric],
) -> Vec<GlobalPostmortemClaim> {
    let mut out: Vec<GlobalPostmortemClaim> = Vec::new();
    for item in items.iter() {
        let derived = new_global_postmortem_claim(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn resolve_global_postmortem_claim(item: &GlobalPostmortemClaim) -> f64 {
    action_item_score::adaptive_action_item_score_weight(5, true)
}
