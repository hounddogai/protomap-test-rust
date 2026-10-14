use super::*;

use std::collections::BTreeMap;

use crate::measurement::cycle as measurement_cycle;

pub fn summarize_pending_measurement_cycles_for_record(
    items: &[measurement_cycle::PendingMeasurementCycle],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_exposure();
    }
    totals
}

pub fn derive_fallback_measurement_records_from_cycle(
    items: &[measurement_cycle::PendingMeasurementCycle],
) -> Vec<FallbackMeasurementRecord> {
    let mut out: Vec<FallbackMeasurementRecord> = Vec::new();
    for item in items.iter() {
        let derived = new_fallback_measurement_record(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn settle_fallback_measurement_record(item: &FallbackMeasurementRecord) -> String {
    measurement_cycle::classify_measurement_tier(item.effective_value())
}
