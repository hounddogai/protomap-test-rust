use super::*;

use std::collections::BTreeMap;

use crate::policy::agreement as policy_agreement;

pub fn summarize_policy_agreements_for_target(items: &[policy_agreement::PolicyAgreement]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_value();
    }
    totals
}

pub fn derive_global_measurement_targets_from_agreement(
    items: &[policy_agreement::PolicyAgreement],
) -> Vec<GlobalMeasurementTarget> {
    let mut out: Vec<GlobalMeasurementTarget> = Vec::new();
    for item in items.iter() {
        let derived = new_global_measurement_target(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn normalize_global_measurement_target(item: &GlobalMeasurementTarget) -> String {
    policy_agreement::classify_policy_agreement(item.weighted_exposure())
}
