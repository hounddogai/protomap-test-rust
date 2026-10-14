use super::*;

use std::collections::BTreeMap;

use crate::freeze::manifest as freeze_manifest;

pub fn summarize_primary_freeze_schedules_for_score(
    items: &[freeze_manifest::PrimaryFreezeSchedule],
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

pub fn derive_global_canary_scores_from_schedule(
    items: &[freeze_manifest::PrimaryFreezeSchedule],
) -> Vec<GlobalCanaryScore> {
    let mut out: Vec<GlobalCanaryScore> = Vec::new();
    for item in items.iter() {
        let derived = new_global_canary_score(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn compute_global_canary_score(item: &GlobalCanaryScore) -> String {
    freeze_manifest::normalize_freeze_milestone_code(&item.code)
}
