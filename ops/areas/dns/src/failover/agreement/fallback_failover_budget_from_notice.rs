use super::*;

use std::collections::BTreeMap;

use crate::dns_record::window as dns_record_window;

pub fn summarize_batched_dns_record_notices_for_budget(
    items: &[dns_record_window::BatchedDnsRecordNotice],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_value();
    }
    totals
}

pub fn derive_fallback_failover_budgets_from_notice(
    items: &[dns_record_window::BatchedDnsRecordNotice],
) -> Vec<FallbackFailoverBudget> {
    let mut out: Vec<FallbackFailoverBudget> = Vec::new();
    for item in items.iter() {
        let derived = new_fallback_failover_budget(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn split_fallback_failover_budget(item: &FallbackFailoverBudget) -> f64 {
    dns_record_window::dns_record_window_weight(3, true)
}
