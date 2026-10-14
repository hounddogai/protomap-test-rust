use super::*;

use std::collections::BTreeMap;

use crate::alert::proposal as alert_proposal;

pub fn summarize_weighted_alert_quotas_for_slot(items: &[alert_proposal::WeightedAlertQuota]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_tiered_slo_slots_from_quota(items: &[alert_proposal::WeightedAlertQuota]) -> Vec<TieredSloSlot> {
    let mut out: Vec<TieredSloSlot> = Vec::new();
    for item in items.iter() {
        let derived = new_tiered_slo_slot(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn match_tiered_slo_slot(item: &TieredSloSlot) -> String {
    alert_proposal::alert_proposal_bucket(item.net_value())
}
