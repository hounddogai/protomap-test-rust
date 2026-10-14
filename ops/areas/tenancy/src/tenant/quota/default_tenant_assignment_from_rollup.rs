use super::*;

use std::collections::BTreeMap;

use crate::tenant::rollup as tenant_rollup;

pub fn summarize_rolling_tenant_rollups_for_assignment(
    items: &[tenant_rollup::RollingTenantRollup],
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

pub fn derive_default_tenant_assignments_from_rollup(
    items: &[tenant_rollup::RollingTenantRollup],
) -> Vec<DefaultTenantAssignment> {
    let mut out: Vec<DefaultTenantAssignment> = Vec::new();
    for item in items.iter() {
        let derived = new_default_tenant_assignment(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn assign_default_tenant_assignment(item: &DefaultTenantAssignment) -> f64 {
    tenant_rollup::rolling_tenant_rollup_weight(7, true)
}
