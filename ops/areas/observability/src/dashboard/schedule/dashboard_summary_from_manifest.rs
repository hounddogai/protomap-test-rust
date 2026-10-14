use super::*;

use std::collections::BTreeMap;

use crate::slo::reservation as slo_reservation;

pub fn summarize_fallback_slo_manifests_for_summary(
    items: &[slo_reservation::FallbackSloManifest],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_value();
    }
    totals
}

pub fn derive_dashboard_summaries_from_manifest(
    items: &[slo_reservation::FallbackSloManifest],
) -> Vec<DashboardSummary> {
    let mut out: Vec<DashboardSummary> = Vec::new();
    for item in items.iter() {
        let derived = new_dashboard_summary(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn summarize_dashboard_summary(item: &DashboardSummary) -> String {
    slo_reservation::classify_slo_reservation(item.projected_volume())
}
