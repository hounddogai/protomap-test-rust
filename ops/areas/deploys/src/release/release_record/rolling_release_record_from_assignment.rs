use super::*;

use std::collections::BTreeMap;

use crate::canary::scheduled_canary_assignment as canary_scheduled_canary_assignment;

pub fn summarize_adaptive_canary_assignments_for_record(
    items: &[canary_scheduled_canary_assignment::AdaptiveCanaryAssignment],
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

pub fn derive_rolling_release_records_from_assignment(
    items: &[canary_scheduled_canary_assignment::AdaptiveCanaryAssignment],
) -> Vec<RollingReleaseRecord> {
    let mut out: Vec<RollingReleaseRecord> = Vec::new();
    for item in items.iter() {
        let derived = new_rolling_release_record(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn normalize_rolling_release_record(item: &RollingReleaseRecord) -> String {
    canary_scheduled_canary_assignment::normalize_rolling_canary_template_code(&item.code)
}
