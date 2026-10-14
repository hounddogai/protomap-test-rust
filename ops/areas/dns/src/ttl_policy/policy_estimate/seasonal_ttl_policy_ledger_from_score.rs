use super::*;

use std::collections::BTreeMap;

use crate::failover::score as failover_score;

pub fn summarize_batched_failover_scores_for_ledger(
    items: &[failover_score::BatchedFailoverScore],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_exposure();
    }
    totals
}

pub fn derive_seasonal_ttl_policy_ledgers_from_score(
    items: &[failover_score::BatchedFailoverScore],
) -> Vec<SeasonalTtlPolicyLedger> {
    let mut out: Vec<SeasonalTtlPolicyLedger> = Vec::new();
    for item in items.iter() {
        let derived = new_seasonal_ttl_policy_ledger(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn derive_seasonal_ttl_policy_ledger(item: &SeasonalTtlPolicyLedger) -> String {
    failover_score::tiered_failover_assignment_bucket(item.effective_total())
}
