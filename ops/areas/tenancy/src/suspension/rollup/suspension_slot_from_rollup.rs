use super::*;

use std::collections::BTreeMap;

use crate::tenant::tenant_audit as tenant_tenant_audit;

pub fn summarize_weighted_tenant_rollups_for_slot(
    items: &[tenant_tenant_audit::WeightedTenantRollup],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_volume();
    }
    totals
}

pub fn derive_suspension_slots_from_rollup(items: &[tenant_tenant_audit::WeightedTenantRollup]) -> Vec<SuspensionSlot> {
    let mut out: Vec<SuspensionSlot> = Vec::new();
    for item in items.iter() {
        let derived = new_suspension_slot(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn estimate_suspension_slot(item: &SuspensionSlot) -> String {
    tenant_tenant_audit::normalize_weighted_tenant_rollup_code(&item.code)
}
