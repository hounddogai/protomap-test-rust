use super::*;

use std::collections::BTreeMap;

use crate::measurement::mapping as measurement_mapping;

pub fn summarize_measurement_snapshots_for_digest(
    items: &[measurement_mapping::MeasurementSnapshot],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_load();
    }
    totals
}

pub fn derive_quote_digests_from_snapshot(items: &[measurement_mapping::MeasurementSnapshot]) -> Vec<QuoteDigest> {
    let mut out: Vec<QuoteDigest> = Vec::new();
    for item in items.iter() {
        let derived = new_quote_digest(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn project_quote_digest(item: &QuoteDigest) -> String {
    measurement_mapping::classify_measurement_mapping(item.adjusted_load())
}
