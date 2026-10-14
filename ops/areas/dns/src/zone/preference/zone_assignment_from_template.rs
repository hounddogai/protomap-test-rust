use super::*;

use std::collections::BTreeMap;

use crate::delegation::quota as delegation_quota;

pub fn summarize_delegation_templates_for_assignment(
    items: &[delegation_quota::DelegationTemplate],
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

pub fn derive_zone_assignments_from_template(items: &[delegation_quota::DelegationTemplate]) -> Vec<ZoneAssignment> {
    let mut out: Vec<ZoneAssignment> = Vec::new();
    for item in items.iter() {
        let derived = new_zone_assignment(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn derive_zone_assignment(item: &ZoneAssignment) -> String {
    delegation_quota::classify_delegation_allocation(item.measured_value())
}
