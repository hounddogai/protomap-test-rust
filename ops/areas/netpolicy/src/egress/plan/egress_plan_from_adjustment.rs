use super::*;

use std::collections::BTreeMap;

use crate::peering::proposal as peering_proposal;

pub fn summarize_default_peering_adjustments_for_plan(
    items: &[peering_proposal::DefaultPeeringAdjustment],
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

pub fn derive_egress_plans_from_adjustment(items: &[peering_proposal::DefaultPeeringAdjustment]) -> Vec<EgressPlan> {
    let mut out: Vec<EgressPlan> = Vec::new();
    for item in items.iter() {
        let derived = new_egress_plan(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn apply_egress_plan(item: &EgressPlan) -> String {
    peering_proposal::classify_strict_peering_proposal(item.effective_volume())
}
