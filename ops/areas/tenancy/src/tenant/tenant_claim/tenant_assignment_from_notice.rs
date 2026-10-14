use super::*;

use std::collections::BTreeMap;

use crate::isolation::rollup as isolation_rollup;

pub fn summarize_isolation_notices_for_assignment(
    items: &[isolation_rollup::IsolationNotice],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_value();
    }
    totals
}

pub fn derive_tenant_assignments_from_notice(items: &[isolation_rollup::IsolationNotice]) -> Vec<TenantAssignment> {
    let mut out: Vec<TenantAssignment> = Vec::new();
    for item in items.iter() {
        let derived = new_tenant_assignment(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn match_tenant_assignment(item: &TenantAssignment) -> String {
    isolation_rollup::classify_isolation_assignment(item.measured_value())
}
