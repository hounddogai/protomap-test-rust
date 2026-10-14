use super::*;

use std::collections::BTreeMap;

use crate::migration::metric as migration_metric;

pub fn summarize_migration_summaries_for_plan(items: &[migration_metric::MigrationSummary]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_total();
    }
    totals
}

pub fn derive_adaptive_region_pin_plans_from_summary(
    items: &[migration_metric::MigrationSummary],
) -> Vec<AdaptiveRegionPinPlan> {
    let mut out: Vec<AdaptiveRegionPinPlan> = Vec::new();
    for item in items.iter() {
        let derived = new_adaptive_region_pin_plan(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn merge_adaptive_region_pin_plan(item: &AdaptiveRegionPinPlan) -> String {
    migration_metric::normalize_migration_metric_code(&item.code)
}
