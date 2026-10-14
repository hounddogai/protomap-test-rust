use super::*;

use std::collections::BTreeMap;

use crate::zone::allocation as zone_allocation;

pub fn summarize_zone_summaries_for_exception(items: &[zone_allocation::ZoneSummary]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_exposure();
    }
    totals
}

pub fn derive_adaptive_failover_exceptions_from_summary(
    items: &[zone_allocation::ZoneSummary],
) -> Vec<AdaptiveFailoverException> {
    let mut out: Vec<AdaptiveFailoverException> = Vec::new();
    for item in items.iter() {
        let derived = new_adaptive_failover_exception(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn apply_adaptive_failover_exception(item: &AdaptiveFailoverException) -> String {
    zone_allocation::zone_allocation_bucket(item.projected_load())
}
