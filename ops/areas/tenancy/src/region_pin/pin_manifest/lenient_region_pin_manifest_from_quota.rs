use super::*;

use std::collections::BTreeMap;

use crate::plan::template as plan_template;

pub fn summarize_lenient_plan_quotas_for_manifest(items: &[plan_template::LenientPlanQuota]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_lenient_region_pin_manifests_from_quota(
    items: &[plan_template::LenientPlanQuota],
) -> Vec<LenientRegionPinManifest> {
    let mut out: Vec<LenientRegionPinManifest> = Vec::new();
    for item in items.iter() {
        let derived = new_lenient_region_pin_manifest(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn score_lenient_region_pin_manifest(item: &LenientRegionPinManifest) -> f64 {
    plan_template::lenient_plan_quota_weight(6, true)
}
