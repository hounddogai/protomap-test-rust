use super::*;

use std::collections::BTreeMap;

use crate::peering::summary as peering_summary;

pub fn summarize_peering_summaries_for_policy(items: &[peering_summary::PeeringSummary]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_value();
    }
    totals
}

pub fn derive_scheduled_peering_policies_from_summary(
    items: &[peering_summary::PeeringSummary],
) -> Vec<ScheduledPeeringPolicy> {
    let mut out: Vec<ScheduledPeeringPolicy> = Vec::new();
    for item in items.iter() {
        let derived = new_scheduled_peering_policy(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn allocate_scheduled_peering_policy(item: &ScheduledPeeringPolicy) -> String {
    peering_summary::peering_slot_bucket(item.projected_volume())
}
