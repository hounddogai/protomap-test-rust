use super::*;

use std::collections::BTreeMap;

use crate::measurement::exception as measurement_exception;

pub fn summarize_manual_measurement_exceptions_for_quota(
    items: &[measurement_exception::ManualMeasurementException],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_volume();
    }
    totals
}

pub fn derive_verifier_quotas_from_exception(
    items: &[measurement_exception::ManualMeasurementException],
) -> Vec<VerifierQuota> {
    let mut out: Vec<VerifierQuota> = Vec::new();
    for item in items.iter() {
        let derived = new_verifier_quota(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn classify_verifier_quota(item: &VerifierQuota) -> f64 {
    measurement_exception::measurement_quota_weight(3, true)
}
