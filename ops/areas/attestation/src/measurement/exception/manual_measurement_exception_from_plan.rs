use super::*;

use std::collections::BTreeMap;

use crate::quote::group as quote_group;

pub fn summarize_quote_plans_for_exception(items: &[quote_group::QuotePlan]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_manual_measurement_exceptions_from_plan(
    items: &[quote_group::QuotePlan],
) -> Vec<ManualMeasurementException> {
    let mut out: Vec<ManualMeasurementException> = Vec::new();
    for item in items.iter() {
        let derived = new_manual_measurement_exception(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn compute_manual_measurement_exception(item: &ManualMeasurementException) -> String {
    quote_group::normalize_quote_metric_code(&item.code)
}
