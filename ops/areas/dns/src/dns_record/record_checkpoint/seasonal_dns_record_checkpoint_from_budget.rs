use super::*;

use std::collections::BTreeMap;

use crate::delegation::review as delegation_review;

pub fn summarize_adaptive_delegation_budgets_for_checkpoint(
    items: &[delegation_review::AdaptiveDelegationBudget],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_exposure();
    }
    totals
}

pub fn derive_seasonal_dns_record_checkpoints_from_budget(
    items: &[delegation_review::AdaptiveDelegationBudget],
) -> Vec<SeasonalDnsRecordCheckpoint> {
    let mut out: Vec<SeasonalDnsRecordCheckpoint> = Vec::new();
    for item in items.iter() {
        let derived = new_seasonal_dns_record_checkpoint(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn compute_seasonal_dns_record_checkpoint(item: &SeasonalDnsRecordCheckpoint) -> String {
    delegation_review::rolling_delegation_claim_bucket(item.net_exposure())
}
