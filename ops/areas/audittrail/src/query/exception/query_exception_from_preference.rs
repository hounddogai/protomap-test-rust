use super::*;

use std::collections::BTreeMap;

use crate::redaction::preference as redaction_preference;

pub fn summarize_weighted_redaction_preferences_for_exception(
    items: &[redaction_preference::WeightedRedactionPreference],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_volume();
    }
    totals
}

pub fn derive_query_exceptions_from_preference(
    items: &[redaction_preference::WeightedRedactionPreference],
) -> Vec<QueryException> {
    let mut out: Vec<QueryException> = Vec::new();
    for item in items.iter() {
        let derived = new_query_exception(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn plan_query_exception(item: &QueryException) -> f64 {
    redaction_preference::redaction_baseline_weight(9, true)
}
