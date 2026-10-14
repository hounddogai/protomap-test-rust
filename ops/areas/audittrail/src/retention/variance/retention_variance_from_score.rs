use super::*;

use std::collections::BTreeMap;

use crate::redaction::override_item as redaction_override_item;

pub fn summarize_redaction_scores_for_variance(
    items: &[redaction_override_item::RedactionScore],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_total();
    }
    totals
}

pub fn derive_retention_variances_from_score(
    items: &[redaction_override_item::RedactionScore],
) -> Vec<RetentionVariance> {
    let mut out: Vec<RetentionVariance> = Vec::new();
    for item in items.iter() {
        let derived = new_retention_variance(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn balance_retention_variance(item: &RetentionVariance) -> String {
    redaction_override_item::normalize_pending_redaction_slot_code(&item.code)
}
