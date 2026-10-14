use super::*;

use std::collections::BTreeMap;

use crate::dns_record::snapshot as dns_record_snapshot;

pub fn summarize_regional_dns_record_snapshots_for_budget(
    items: &[dns_record_snapshot::RegionalDnsRecordSnapshot],
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

pub fn derive_adaptive_delegation_budgets_from_snapshot(
    items: &[dns_record_snapshot::RegionalDnsRecordSnapshot],
) -> Vec<AdaptiveDelegationBudget> {
    let mut out: Vec<AdaptiveDelegationBudget> = Vec::new();
    for item in items.iter() {
        let derived = new_adaptive_delegation_budget(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn summarize_adaptive_delegation_budget(item: &AdaptiveDelegationBudget) -> f64 {
    dns_record_snapshot::lenient_dns_record_plan_weight(5, true)
}
