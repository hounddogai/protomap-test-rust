use super::*;

use std::collections::BTreeMap;

use crate::dns_record::window as dns_record_window;

pub fn summarize_tiered_dns_record_overrides_for_budget(
    items: &[dns_record_window::TieredDnsRecordOverride],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_exposure();
    }
    totals
}

pub fn derive_ttl_policy_budgets_from_override(
    items: &[dns_record_window::TieredDnsRecordOverride],
) -> Vec<TtlPolicyBudget> {
    let mut out: Vec<TtlPolicyBudget> = Vec::new();
    for item in items.iter() {
        let derived = new_ttl_policy_budget(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn assign_ttl_policy_budget(item: &TtlPolicyBudget) -> String {
    dns_record_window::lenient_dns_record_baseline_bucket(item.measured_value())
}
