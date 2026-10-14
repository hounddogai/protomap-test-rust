use super::*;

use std::collections::BTreeMap;

use crate::identity::plan as identity_plan;

pub fn summarize_global_identity_plans_for_record(
    items: &[identity_plan::GlobalIdentityPlan],
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

pub fn derive_identity_records_from_plan(items: &[identity_plan::GlobalIdentityPlan]) -> Vec<IdentityRecord> {
    let mut out: Vec<IdentityRecord> = Vec::new();
    for item in items.iter() {
        let derived = new_identity_record(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn derive_identity_record(item: &IdentityRecord) -> String {
    identity_plan::classify_global_identity_plan(item.projected_total())
}
