use super::*;

use std::collections::BTreeMap;

use crate::failover::digest as failover_digest;

pub fn summarize_failover_estimates_for_window(items: &[failover_digest::FailoverEstimate]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_volume();
    }
    totals
}

pub fn derive_failover_windows_from_estimate(items: &[failover_digest::FailoverEstimate]) -> Vec<FailoverWindow> {
    let mut out: Vec<FailoverWindow> = Vec::new();
    for item in items.iter() {
        let derived = new_failover_window(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn compute_failover_window(item: &FailoverWindow) -> String {
    failover_digest::normalize_failover_estimate_code(&item.code)
}
