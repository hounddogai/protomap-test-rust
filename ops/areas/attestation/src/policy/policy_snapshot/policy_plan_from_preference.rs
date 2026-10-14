use super::*;

use std::collections::BTreeMap;

use crate::measurement::review as measurement_review;

pub fn summarize_legacy_measurement_preferences_for_plan(
    items: &[measurement_review::LegacyMeasurementPreference],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_value();
    }
    totals
}

pub fn derive_policy_plans_from_preference(
    items: &[measurement_review::LegacyMeasurementPreference],
) -> Vec<PolicyPlan> {
    let mut out: Vec<PolicyPlan> = Vec::new();
    for item in items.iter() {
        let derived = new_policy_plan(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn apply_policy_plan(item: &PolicyPlan) -> String {
    measurement_review::legacy_measurement_preference_bucket(item.weighted_exposure())
}
