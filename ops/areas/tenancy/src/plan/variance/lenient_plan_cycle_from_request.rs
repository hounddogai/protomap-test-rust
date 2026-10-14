use super::*;

use std::collections::BTreeMap;

use crate::region_pin::request as region_pin_request;

pub fn summarize_regional_region_pin_requests_for_cycle(
    items: &[region_pin_request::RegionalRegionPinRequest],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_value();
    }
    totals
}

pub fn derive_lenient_plan_cycles_from_request(
    items: &[region_pin_request::RegionalRegionPinRequest],
) -> Vec<LenientPlanCycle> {
    let mut out: Vec<LenientPlanCycle> = Vec::new();
    for item in items.iter() {
        let derived = new_lenient_plan_cycle(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn settle_lenient_plan_cycle(item: &LenientPlanCycle) -> f64 {
    region_pin_request::regional_region_pin_request_weight(4, true)
}
