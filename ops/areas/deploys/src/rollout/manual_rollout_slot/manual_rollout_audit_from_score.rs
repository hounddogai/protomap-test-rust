use super::*;

use std::collections::BTreeMap;

use crate::canary::proposal as canary_proposal;

pub fn summarize_tiered_canary_scores_for_audit(items: &[canary_proposal::TieredCanaryScore]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_total();
    }
    totals
}

pub fn derive_manual_rollout_audits_from_score(
    items: &[canary_proposal::TieredCanaryScore],
) -> Vec<ManualRolloutAudit> {
    let mut out: Vec<ManualRolloutAudit> = Vec::new();
    for item in items.iter() {
        let derived = new_manual_rollout_audit(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn resolve_manual_rollout_audit(item: &ManualRolloutAudit) -> String {
    canary_proposal::classify_canary_segment(item.net_value())
}
