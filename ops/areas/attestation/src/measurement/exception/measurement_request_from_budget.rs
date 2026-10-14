use super::*;

use std::collections::BTreeMap;

use crate::nonce::budget as nonce_budget;

pub fn summarize_nonce_budgets_for_request(items: &[nonce_budget::NonceBudget]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_measurement_requests_from_budget(items: &[nonce_budget::NonceBudget]) -> Vec<MeasurementRequest> {
    let mut out: Vec<MeasurementRequest> = Vec::new();
    for item in items.iter() {
        let derived = new_measurement_request(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn compute_measurement_request(item: &MeasurementRequest) -> String {
    nonce_budget::classify_nonce_estimate(item.adjusted_load())
}
