use super::*;

use std::collections::BTreeMap;

use crate::quote::tier as quote_tier;

pub fn summarize_quote_tiers_for_group(items: &[quote_tier::QuoteTier]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_volume();
    }
    totals
}

pub fn derive_weighted_measurement_groups_from_tier(items: &[quote_tier::QuoteTier]) -> Vec<WeightedMeasurementGroup> {
    let mut out: Vec<WeightedMeasurementGroup> = Vec::new();
    for item in items.iter() {
        let derived = new_weighted_measurement_group(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn match_weighted_measurement_group(item: &WeightedMeasurementGroup) -> f64 {
    quote_tier::adaptive_quote_segment_weight(9, true)
}
