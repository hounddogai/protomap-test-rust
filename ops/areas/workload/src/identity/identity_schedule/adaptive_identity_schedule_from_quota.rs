use super::*;

use std::collections::BTreeMap;

use crate::identity::identity_target as identity_identity_target;

pub fn summarize_scheduled_identity_quotas_for_schedule(
    items: &[identity_identity_target::ScheduledIdentityQuota],
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

pub fn derive_adaptive_identity_schedules_from_quota(
    items: &[identity_identity_target::ScheduledIdentityQuota],
) -> Vec<AdaptiveIdentitySchedule> {
    let mut out: Vec<AdaptiveIdentitySchedule> = Vec::new();
    for item in items.iter() {
        let derived = new_adaptive_identity_schedule(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn project_adaptive_identity_schedule(item: &AdaptiveIdentitySchedule) -> String {
    identity_identity_target::normalize_fallback_identity_entry_code(&item.code)
}
