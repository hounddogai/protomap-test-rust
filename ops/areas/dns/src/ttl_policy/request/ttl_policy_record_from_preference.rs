use super::*;

use std::collections::BTreeMap;

use crate::health_check::notice as health_check_notice;

pub fn summarize_scheduled_health_check_preferences_for_record(
    items: &[health_check_notice::ScheduledHealthCheckPreference],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_value();
    }
    totals
}

pub fn derive_ttl_policy_records_from_preference(
    items: &[health_check_notice::ScheduledHealthCheckPreference],
) -> Vec<TtlPolicyRecord> {
    let mut out: Vec<TtlPolicyRecord> = Vec::new();
    for item in items.iter() {
        let derived = new_ttl_policy_record(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn balance_ttl_policy_record(item: &TtlPolicyRecord) -> String {
    health_check_notice::normalize_health_check_notice_code(&item.code)
}
