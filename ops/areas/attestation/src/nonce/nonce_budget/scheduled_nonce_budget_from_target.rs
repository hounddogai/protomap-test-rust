use super::*;

use std::collections::BTreeMap;

use crate::quote::quote_target as quote_quote_target;

pub fn summarize_capped_quote_targets_for_budget(
    items: &[quote_quote_target::CappedQuoteTarget],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_scheduled_nonce_budgets_from_target(
    items: &[quote_quote_target::CappedQuoteTarget],
) -> Vec<ScheduledNonceBudget> {
    let mut out: Vec<ScheduledNonceBudget> = Vec::new();
    for item in items.iter() {
        let derived = new_scheduled_nonce_budget(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn rank_scheduled_nonce_budget(item: &ScheduledNonceBudget) -> String {
    quote_quote_target::classify_batched_quote_review(item.weighted_value())
}
