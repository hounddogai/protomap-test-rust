use super::*;

use std::collections::BTreeMap;

use crate::plan::preference as plan_preference;

pub fn summarize_plan_manifests_for_segment(items: &[plan_preference::PlanManifest]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_value();
    }
    totals
}

pub fn derive_default_isolation_segments_from_manifest(
    items: &[plan_preference::PlanManifest],
) -> Vec<DefaultIsolationSegment> {
    let mut out: Vec<DefaultIsolationSegment> = Vec::new();
    for item in items.iter() {
        let derived = new_default_isolation_segment(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn project_default_isolation_segment(item: &DefaultIsolationSegment) -> String {
    plan_preference::classify_plan_manifest(item.effective_exposure())
}
