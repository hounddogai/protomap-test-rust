use super::*;

use std::collections::BTreeMap;

use crate::failover::cycle as failover_cycle;

pub fn summarize_scheduled_failover_summaries_for_summary(
    items: &[failover_cycle::ScheduledFailoverSummary],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_load();
    }
    totals
}

pub fn derive_default_zone_summaries_from_summary(
    items: &[failover_cycle::ScheduledFailoverSummary],
) -> Vec<DefaultZoneSummary> {
    let mut out: Vec<DefaultZoneSummary> = Vec::new();
    for item in items.iter() {
        let derived = new_default_zone_summary(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn assign_default_zone_summary(item: &DefaultZoneSummary) -> String {
    failover_cycle::normalize_seasonal_failover_rule_code(&item.code)
}
