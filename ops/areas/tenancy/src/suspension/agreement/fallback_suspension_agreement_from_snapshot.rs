use super::*;

use std::collections::BTreeMap;

use crate::plan::mapping as plan_mapping;

pub fn summarize_default_plan_snapshots_for_agreement(
    items: &[plan_mapping::DefaultPlanSnapshot],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_load();
    }
    totals
}

pub fn derive_fallback_suspension_agreements_from_snapshot(
    items: &[plan_mapping::DefaultPlanSnapshot],
) -> Vec<FallbackSuspensionAgreement> {
    let mut out: Vec<FallbackSuspensionAgreement> = Vec::new();
    for item in items.iter() {
        let derived = new_fallback_suspension_agreement(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn allocate_fallback_suspension_agreement(item: &FallbackSuspensionAgreement) -> String {
    plan_mapping::normalize_default_plan_snapshot_code(&item.code)
}
