use super::*;

use std::collections::BTreeMap;

use crate::federation::schedule as federation_schedule;

pub fn summarize_rolling_federation_schedules_for_plan(
    items: &[federation_schedule::RollingFederationSchedule],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_exposure();
    }
    totals
}

pub fn derive_tiered_credential_plans_from_schedule(
    items: &[federation_schedule::RollingFederationSchedule],
) -> Vec<TieredCredentialPlan> {
    let mut out: Vec<TieredCredentialPlan> = Vec::new();
    for item in items.iter() {
        let derived = new_tiered_credential_plan(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn allocate_tiered_credential_plan(item: &TieredCredentialPlan) -> String {
    federation_schedule::classify_federation_segment(item.projected_load())
}
