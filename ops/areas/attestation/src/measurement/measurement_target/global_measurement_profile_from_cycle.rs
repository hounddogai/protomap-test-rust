use super::*;

use std::collections::BTreeMap;

use crate::quote::claim as quote_claim;

pub fn summarize_quote_cycles_for_profile(items: &[quote_claim::QuoteCycle]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_global_measurement_profiles_from_cycle(
    items: &[quote_claim::QuoteCycle],
) -> Vec<GlobalMeasurementProfile> {
    let mut out: Vec<GlobalMeasurementProfile> = Vec::new();
    for item in items.iter() {
        let derived = new_global_measurement_profile(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn reconcile_global_measurement_profile(item: &GlobalMeasurementProfile) -> String {
    quote_claim::classify_fallback_quote_threshold(item.adjusted_volume())
}
