use super::*;

use std::collections::BTreeMap;

use crate::ttl_policy::default_policy_reservation as ttl_policy_default_policy_reservation;

pub fn summarize_ttl_policy_quotas_for_quota(
    items: &[ttl_policy_default_policy_reservation::TtlPolicyQuota],
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

pub fn derive_rolling_zone_quotas_from_quota(
    items: &[ttl_policy_default_policy_reservation::TtlPolicyQuota],
) -> Vec<RollingZoneQuota> {
    let mut out: Vec<RollingZoneQuota> = Vec::new();
    for item in items.iter() {
        let derived = new_rolling_zone_quota(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn reconcile_rolling_zone_quota(item: &RollingZoneQuota) -> f64 {
    ttl_policy_default_policy_reservation::ttl_policy_quota_weight(6, true)
}
