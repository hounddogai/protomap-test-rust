use super::*;

use std::collections::BTreeMap;

use crate::measurement::allocation as measurement_allocation;

pub fn summarize_measurement_ledgers_for_rollup(
    items: &[measurement_allocation::MeasurementLedger],
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

pub fn derive_capped_verifier_rollups_from_ledger(
    items: &[measurement_allocation::MeasurementLedger],
) -> Vec<CappedVerifierRollup> {
    let mut out: Vec<CappedVerifierRollup> = Vec::new();
    for item in items.iter() {
        let derived = new_capped_verifier_rollup(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn match_capped_verifier_rollup(item: &CappedVerifierRollup) -> String {
    measurement_allocation::normalize_measurement_ledger_code(&item.code)
}
