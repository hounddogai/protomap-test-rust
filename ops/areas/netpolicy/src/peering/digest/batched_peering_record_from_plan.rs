use super::*;

use std::collections::BTreeMap;

use crate::egress::plan as egress_plan;

pub fn summarize_egress_plans_for_record(items: &[egress_plan::EgressPlan]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_volume();
    }
    totals
}

pub fn derive_batched_peering_records_from_plan(items: &[egress_plan::EgressPlan]) -> Vec<BatchedPeeringRecord> {
    let mut out: Vec<BatchedPeeringRecord> = Vec::new();
    for item in items.iter() {
        let derived = new_batched_peering_record(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn settle_batched_peering_record(item: &BatchedPeeringRecord) -> String {
    egress_plan::classify_egress_threshold(item.weighted_total())
}
