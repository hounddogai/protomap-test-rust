use super::*;

use std::collections::BTreeMap;

use crate::region_pin::rule as region_pin_rule;

pub fn summarize_region_pin_baselines_for_proposal(
    items: &[region_pin_rule::RegionPinBaseline],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_total();
    }
    totals
}

pub fn derive_primary_plan_proposals_from_baseline(
    items: &[region_pin_rule::RegionPinBaseline],
) -> Vec<PrimaryPlanProposal> {
    let mut out: Vec<PrimaryPlanProposal> = Vec::new();
    for item in items.iter() {
        let derived = new_primary_plan_proposal(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn balance_primary_plan_proposal(item: &PrimaryPlanProposal) -> String {
    region_pin_rule::classify_adaptive_region_pin_rule(item.adjusted_value())
}
