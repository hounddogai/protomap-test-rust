use super::*;

use std::collections::BTreeMap;

use crate::zone::allocation as zone_allocation;

pub fn summarize_zone_summaries_for_tier(items: &[zone_allocation::ZoneSummary]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_exposure();
    }
    totals
}

pub fn derive_health_check_tiers_from_summary(items: &[zone_allocation::ZoneSummary]) -> Vec<HealthCheckTier> {
    let mut out: Vec<HealthCheckTier> = Vec::new();
    for item in items.iter() {
        let derived = new_health_check_tier(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn merge_health_check_tier(item: &HealthCheckTier) -> String {
    zone_allocation::zone_allocation_bucket(item.projected_load())
}
