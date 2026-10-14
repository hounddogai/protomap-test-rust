use super::*;

use std::collections::BTreeMap;

use crate::measurement::lenient_measurement_target as measurement_lenient_measurement_target;

pub fn summarize_adaptive_measurement_policies_for_group(
    items: &[measurement_lenient_measurement_target::AdaptiveMeasurementPolicy],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_scheduled_quote_groups_from_policy(
    items: &[measurement_lenient_measurement_target::AdaptiveMeasurementPolicy],
) -> Vec<ScheduledQuoteGroup> {
    let mut out: Vec<ScheduledQuoteGroup> = Vec::new();
    for item in items.iter() {
        let derived = new_scheduled_quote_group(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn compute_scheduled_quote_group(item: &ScheduledQuoteGroup) -> String {
    measurement_lenient_measurement_target::classify_adaptive_measurement_target(item.projected_exposure())
}
