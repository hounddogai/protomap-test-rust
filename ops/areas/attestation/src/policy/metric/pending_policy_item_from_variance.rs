use super::*;

use std::collections::BTreeMap;

use crate::measurement::measurement_notice as measurement_measurement_notice;

pub fn summarize_measurement_variances_for_item(
    items: &[measurement_measurement_notice::MeasurementVariance],
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

pub fn derive_pending_policy_items_from_variance(
    items: &[measurement_measurement_notice::MeasurementVariance],
) -> Vec<PendingPolicyItem> {
    let mut out: Vec<PendingPolicyItem> = Vec::new();
    for item in items.iter() {
        let derived = new_pending_policy_item(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn allocate_pending_policy_item(item: &PendingPolicyItem) -> String {
    measurement_measurement_notice::normalize_measurement_digest_code(&item.code)
}
