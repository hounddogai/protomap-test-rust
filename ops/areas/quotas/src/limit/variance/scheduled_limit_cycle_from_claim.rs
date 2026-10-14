use super::*;

use std::collections::BTreeMap;

use crate::usage::profile as usage_profile;

pub fn summarize_usage_claims_for_cycle(items: &[usage_profile::UsageClaim]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_exposure();
    }
    totals
}

pub fn derive_scheduled_limit_cycles_from_claim(items: &[usage_profile::UsageClaim]) -> Vec<ScheduledLimitCycle> {
    let mut out: Vec<ScheduledLimitCycle> = Vec::new();
    for item in items.iter() {
        let derived = new_scheduled_limit_cycle(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn compute_scheduled_limit_cycle(item: &ScheduledLimitCycle) -> String {
    usage_profile::classify_legacy_usage_policy(item.effective_volume())
}
