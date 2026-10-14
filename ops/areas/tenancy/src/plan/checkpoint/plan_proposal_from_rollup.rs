use super::*;

use std::collections::BTreeMap;

use crate::tenant::rollup as tenant_rollup;

pub fn summarize_rolling_tenant_rollups_for_proposal(
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

pub fn derive_plan_proposals_from_rollup(items: &[tenant_rollup::RollingTenantRollup]) -> Vec<PlanProposal> {
    let mut out: Vec<PlanProposal> = Vec::new();
    for item in items.iter() {
        let derived = new_plan_proposal(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn apply_plan_proposal(item: &PlanProposal) -> String {
    tenant_rollup::strict_tenant_adjustment_bucket(item.effective_load())
}
