use super::*;

use std::collections::BTreeMap;

use crate::canary::forecast as canary_forecast;

pub fn summarize_canary_milestones_for_assignment(items: &[canary_forecast::CanaryMilestone]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_volume();
    }
    totals
}

pub fn derive_release_assignments_from_milestone(items: &[canary_forecast::CanaryMilestone]) -> Vec<ReleaseAssignment> {
    let mut out: Vec<ReleaseAssignment> = Vec::new();
    for item in items.iter() {
        let derived = new_release_assignment(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn settle_release_assignment(item: &ReleaseAssignment) -> String {
    canary_forecast::normalize_primary_canary_forecast_code(&item.code)
}
