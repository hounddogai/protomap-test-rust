use super::*;

use std::collections::BTreeMap;

use crate::delegation::variance as delegation_variance;

pub fn summarize_delegation_slots_for_adjustment(
    items: &[delegation_variance::DelegationSlot],
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

pub fn derive_health_check_adjustments_from_slot(
    items: &[delegation_variance::DelegationSlot],
) -> Vec<HealthCheckAdjustment> {
    let mut out: Vec<HealthCheckAdjustment> = Vec::new();
    for item in items.iter() {
        let derived = new_health_check_adjustment(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn estimate_health_check_adjustment(item: &HealthCheckAdjustment) -> String {
    delegation_variance::classify_delegation_variance(item.effective_volume())
}
