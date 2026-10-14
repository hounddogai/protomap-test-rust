use super::*;

use std::collections::BTreeMap;

use crate::failover::cycle as failover_cycle;

pub fn summarize_capped_failover_preferences_for_template(
    items: &[failover_cycle::CappedFailoverPreference],
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

pub fn derive_weighted_dns_record_templates_from_preference(
    items: &[failover_cycle::CappedFailoverPreference],
) -> Vec<WeightedDnsRecordTemplate> {
    let mut out: Vec<WeightedDnsRecordTemplate> = Vec::new();
    for item in items.iter() {
        let derived = new_weighted_dns_record_template(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn resolve_weighted_dns_record_template(item: &WeightedDnsRecordTemplate) -> String {
    failover_cycle::normalize_capped_failover_preference_code(&item.code)
}
