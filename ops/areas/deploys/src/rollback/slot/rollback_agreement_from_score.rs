use super::*;

use std::collections::BTreeMap;

use crate::canary::policy as canary_policy;

pub fn summarize_canary_scores_for_agreement(items: &[canary_policy::CanaryScore]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_value();
    }
    totals
}

pub fn derive_rollback_agreements_from_score(items: &[canary_policy::CanaryScore]) -> Vec<RollbackAgreement> {
    let mut out: Vec<RollbackAgreement> = Vec::new();
    for item in items.iter() {
        let derived = new_rollback_agreement(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn split_rollback_agreement(item: &RollbackAgreement) -> f64 {
    canary_policy::canary_policy_weight(3, true)
}
