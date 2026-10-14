use super::*;

use std::collections::BTreeMap;

use crate::measurement::measurement_batch as measurement_measurement_batch;

pub fn summarize_tiered_measurement_agreements_for_adjustment(
    items: &[measurement_measurement_batch::TieredMeasurementAgreement],
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

pub fn derive_measurement_adjustments_from_agreement(
    items: &[measurement_measurement_batch::TieredMeasurementAgreement],
) -> Vec<MeasurementAdjustment> {
    let mut out: Vec<MeasurementAdjustment> = Vec::new();
    for item in items.iter() {
        let derived = new_measurement_adjustment(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn allocate_measurement_adjustment(item: &MeasurementAdjustment) -> String {
    measurement_measurement_batch::normalize_primary_measurement_rule_code(&item.code)
}
