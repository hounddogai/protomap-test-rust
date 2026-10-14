use super::*;

use std::collections::BTreeMap;

use crate::escalation::variance as escalation_variance;

pub fn summarize_escalation_variances_for_score(
    items: &[escalation_variance::EscalationVariance],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_value();
    }
    totals
}

pub fn derive_action_item_scores_from_variance(
    items: &[escalation_variance::EscalationVariance],
) -> Vec<ActionItemScore> {
    let mut out: Vec<ActionItemScore> = Vec::new();
    for item in items.iter() {
        let derived = new_action_item_score(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn resolve_action_item_score(item: &ActionItemScore) -> String {
    escalation_variance::classify_escalation_manifest(item.net_exposure())
}
