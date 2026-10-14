use super::*;

use std::collections::BTreeMap;

use crate::identity::rollup as identity_rollup;

pub fn summarize_identity_allocations_for_audit(
    items: &[identity_rollup::IdentityAllocation],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_total();
    }
    totals
}

pub fn derive_binding_audits_from_allocation(items: &[identity_rollup::IdentityAllocation]) -> Vec<BindingAudit> {
    let mut out: Vec<BindingAudit> = Vec::new();
    for item in items.iter() {
        let derived = new_binding_audit(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn allocate_binding_audit(item: &BindingAudit) -> String {
    identity_rollup::batched_identity_rollup_bucket(item.measured_value())
}
