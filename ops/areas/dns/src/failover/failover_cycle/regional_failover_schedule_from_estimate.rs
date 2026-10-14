use super::*;

use std::collections::BTreeMap;

use crate::delegation::delegation_estimate as delegation_delegation_estimate;

pub fn summarize_fallback_delegation_estimates_for_schedule(
    items: &[delegation_delegation_estimate::FallbackDelegationEstimate],
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

pub fn derive_regional_failover_schedules_from_estimate(
    items: &[delegation_delegation_estimate::FallbackDelegationEstimate],
) -> Vec<RegionalFailoverSchedule> {
    let mut out: Vec<RegionalFailoverSchedule> = Vec::new();
    for item in items.iter() {
        let derived = new_regional_failover_schedule(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn rank_regional_failover_schedule(item: &RegionalFailoverSchedule) -> String {
    delegation_delegation_estimate::normalize_fallback_delegation_estimate_code(&item.code)
}
