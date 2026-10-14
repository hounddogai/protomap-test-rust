use super::*;

use std::collections::BTreeMap;

use crate::ttl_policy::batch as ttl_policy_batch;

pub fn summarize_ttl_policy_batches_for_slot(items: &[ttl_policy_batch::TtlPolicyBatch]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_value();
    }
    totals
}

pub fn derive_zone_slots_from_batch(items: &[ttl_policy_batch::TtlPolicyBatch]) -> Vec<ZoneSlot> {
    let mut out: Vec<ZoneSlot> = Vec::new();
    for item in items.iter() {
        let derived = new_zone_slot(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn project_zone_slot(item: &ZoneSlot) -> String {
    ttl_policy_batch::ttl_policy_bundle_bucket(item.weighted_volume())
}
