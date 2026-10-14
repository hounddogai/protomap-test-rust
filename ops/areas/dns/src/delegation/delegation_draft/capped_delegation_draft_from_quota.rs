use super::*;

use std::collections::BTreeMap;

use crate::zone::zone_quota as zone_zone_quota;

pub fn summarize_rolling_zone_quotas_for_draft(items: &[zone_zone_quota::RollingZoneQuota]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_capped_delegation_drafts_from_quota(
    items: &[zone_zone_quota::RollingZoneQuota],
) -> Vec<CappedDelegationDraft> {
    let mut out: Vec<CappedDelegationDraft> = Vec::new();
    for item in items.iter() {
        let derived = new_capped_delegation_draft(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn compute_capped_delegation_draft(item: &CappedDelegationDraft) -> String {
    zone_zone_quota::classify_scheduled_zone_override(item.effective_load())
}
