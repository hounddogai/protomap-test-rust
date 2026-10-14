use super::*;

use std::collections::BTreeMap;

use crate::measurement::record_item as measurement_record_item;

pub fn summarize_tiered_measurement_records_for_record(
    items: &[measurement_record_item::TieredMeasurementRecord],
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

pub fn derive_scheduled_quote_records_from_record(
    items: &[measurement_record_item::TieredMeasurementRecord],
) -> Vec<ScheduledQuoteRecord> {
    let mut out: Vec<ScheduledQuoteRecord> = Vec::new();
    for item in items.iter() {
        let derived = new_scheduled_quote_record(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn split_scheduled_quote_record(item: &ScheduledQuoteRecord) -> String {
    measurement_record_item::fallback_measurement_estimate_bucket(item.measured_value())
}
