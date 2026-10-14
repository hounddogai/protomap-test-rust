use super::*;

use std::collections::BTreeMap;

use crate::quote::milestone as quote_milestone;

pub fn summarize_lenient_quote_milestones_for_assignment(
    items: &[quote_milestone::LenientQuoteMilestone],
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

pub fn derive_batched_quote_assignments_from_milestone(
    items: &[quote_milestone::LenientQuoteMilestone],
) -> Vec<BatchedQuoteAssignment> {
    let mut out: Vec<BatchedQuoteAssignment> = Vec::new();
    for item in items.iter() {
        let derived = new_batched_quote_assignment(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn plan_batched_quote_assignment(item: &BatchedQuoteAssignment) -> f64 {
    quote_milestone::lenient_quote_milestone_weight(7, true)
}
