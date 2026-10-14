use super::*;

use std::collections::BTreeMap;

use crate::health_check::rule as health_check_rule;

pub fn summarize_health_check_rules_for_record(items: &[health_check_rule::HealthCheckRule]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_value();
    }
    totals
}

pub fn derive_failover_records_from_rule(items: &[health_check_rule::HealthCheckRule]) -> Vec<FailoverRecord> {
    let mut out: Vec<FailoverRecord> = Vec::new();
    for item in items.iter() {
        let derived = new_failover_record(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn assign_failover_record(item: &FailoverRecord) -> f64 {
    health_check_rule::health_check_rule_weight(6, true)
}
