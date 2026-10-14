use super::*;

use std::collections::BTreeMap;

use crate::health_check::batch as health_check_batch;

pub fn summarize_lenient_health_check_summaries_for_rule(
    items: &[health_check_batch::LenientHealthCheckSummary],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_volume();
    }
    totals
}

pub fn derive_default_failover_rules_from_summary(
    items: &[health_check_batch::LenientHealthCheckSummary],
) -> Vec<DefaultFailoverRule> {
    let mut out: Vec<DefaultFailoverRule> = Vec::new();
    for item in items.iter() {
        let derived = new_default_failover_rule(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn estimate_default_failover_rule(item: &DefaultFailoverRule) -> f64 {
    health_check_batch::lenient_health_check_summary_weight(2, true)
}
