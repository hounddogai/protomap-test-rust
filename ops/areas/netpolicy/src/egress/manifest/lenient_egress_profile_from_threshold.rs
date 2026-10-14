use super::*;

use std::collections::BTreeMap;

use crate::egress::plan as egress_plan;

pub fn summarize_egress_thresholds_for_profile(items: &[egress_plan::EgressThreshold]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_exposure();
    }
    totals
}

pub fn derive_lenient_egress_profiles_from_threshold(
    items: &[egress_plan::EgressThreshold],
) -> Vec<LenientEgressProfile> {
    let mut out: Vec<LenientEgressProfile> = Vec::new();
    for item in items.iter() {
        let derived = new_lenient_egress_profile(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn split_lenient_egress_profile(item: &LenientEgressProfile) -> f64 {
    egress_plan::seasonal_egress_exception_weight(2, true)
}
