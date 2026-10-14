use super::*;

use std::collections::BTreeMap;

use crate::measurement::measurement_target as measurement_measurement_target;

pub fn summarize_measurement_records_for_profile(
    items: &[measurement_measurement_target::MeasurementRecord],
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

pub fn derive_lenient_quote_profiles_from_record(
    items: &[measurement_measurement_target::MeasurementRecord],
) -> Vec<LenientQuoteProfile> {
    let mut out: Vec<LenientQuoteProfile> = Vec::new();
    for item in items.iter() {
        let derived = new_lenient_quote_profile(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn settle_lenient_quote_profile(item: &LenientQuoteProfile) -> String {
    measurement_measurement_target::classify_measurement_record(item.net_value())
}
