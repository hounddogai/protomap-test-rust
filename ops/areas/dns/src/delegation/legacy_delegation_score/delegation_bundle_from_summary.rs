use super::*;

use std::collections::BTreeMap;

use crate::health_check::rolling_check_milestone as health_check_rolling_check_milestone;

pub fn summarize_strict_health_check_summaries_for_bundle(
    items: &[health_check_rolling_check_milestone::StrictHealthCheckSummary],
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

pub fn derive_delegation_bundles_from_summary(
    items: &[health_check_rolling_check_milestone::StrictHealthCheckSummary],
) -> Vec<DelegationBundle> {
    let mut out: Vec<DelegationBundle> = Vec::new();
    for item in items.iter() {
        let derived = new_delegation_bundle(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn estimate_delegation_bundle(item: &DelegationBundle) -> String {
    health_check_rolling_check_milestone::strict_health_check_milestone_bucket(item.effective_total())
}
