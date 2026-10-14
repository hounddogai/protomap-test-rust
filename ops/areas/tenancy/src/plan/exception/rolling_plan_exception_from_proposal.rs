use super::*;

use std::collections::BTreeMap;

use crate::region_pin::window as region_pin_window;

pub fn summarize_pending_region_pin_proposals_for_exception(
    items: &[region_pin_window::PendingRegionPinProposal],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_exposure();
    }
    totals
}

pub fn derive_rolling_plan_exceptions_from_proposal(
    items: &[region_pin_window::PendingRegionPinProposal],
) -> Vec<RollingPlanException> {
    let mut out: Vec<RollingPlanException> = Vec::new();
    for item in items.iter() {
        let derived = new_rolling_plan_exception(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn resolve_rolling_plan_exception(item: &RollingPlanException) -> String {
    region_pin_window::classify_pending_region_pin_proposal(item.net_load())
}
