use super::*;

use std::collections::BTreeMap;

use crate::failover::variance as failover_variance;

pub fn summarize_failover_variances_for_draft(items: &[failover_variance::FailoverVariance]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_failover_drafts_from_variance(items: &[failover_variance::FailoverVariance]) -> Vec<FailoverDraft> {
    let mut out: Vec<FailoverDraft> = Vec::new();
    for item in items.iter() {
        let derived = new_failover_draft(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn classify_failover_draft(item: &FailoverDraft) -> String {
    failover_variance::failover_slot_bucket(item.measured_value())
}
