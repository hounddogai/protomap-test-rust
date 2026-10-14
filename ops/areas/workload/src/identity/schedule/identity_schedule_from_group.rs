use super::*;

use std::collections::BTreeMap;

use crate::identity::group as identity_group;

pub fn summarize_identity_groups_for_schedule(items: &[identity_group::IdentityGroup]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_value();
    }
    totals
}

pub fn derive_identity_schedules_from_group(items: &[identity_group::IdentityGroup]) -> Vec<IdentitySchedule> {
    let mut out: Vec<IdentitySchedule> = Vec::new();
    for item in items.iter() {
        let derived = new_identity_schedule(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn derive_identity_schedule(item: &IdentitySchedule) -> String {
    identity_group::identity_group_bucket(item.measured_value())
}
