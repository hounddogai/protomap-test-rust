use super::*;

use std::collections::BTreeMap;

use crate::quote::summary as quote_summary;

pub fn summarize_regional_quote_budgets_for_score(
    items: &[quote_summary::RegionalQuoteBudget],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_total();
    }
    totals
}

pub fn derive_scheduled_nonce_scores_from_budget(
    items: &[quote_summary::RegionalQuoteBudget],
) -> Vec<ScheduledNonceScore> {
    let mut out: Vec<ScheduledNonceScore> = Vec::new();
    for item in items.iter() {
        let derived = new_scheduled_nonce_score(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn resolve_scheduled_nonce_score(item: &ScheduledNonceScore) -> String {
    quote_summary::global_quote_cycle_bucket(item.measured_value())
}
