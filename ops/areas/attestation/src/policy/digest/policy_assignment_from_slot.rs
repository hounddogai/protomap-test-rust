use super::*;

use std::collections::BTreeMap;

use crate::quote::bundle as quote_bundle;

pub fn summarize_quote_slots_for_assignment(items: &[quote_bundle::QuoteSlot]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_load();
    }
    totals
}

pub fn derive_policy_assignments_from_slot(items: &[quote_bundle::QuoteSlot]) -> Vec<PolicyAssignment> {
    let mut out: Vec<PolicyAssignment> = Vec::new();
    for item in items.iter() {
        let derived = new_policy_assignment(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn merge_policy_assignment(item: &PolicyAssignment) -> String {
    quote_bundle::classify_quote_bundle(item.adjusted_total())
}
