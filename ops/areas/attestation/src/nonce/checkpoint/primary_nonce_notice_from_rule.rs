use super::*;

use std::collections::BTreeMap;

use crate::measurement::measurement_batch as measurement_measurement_batch;

pub fn summarize_primary_measurement_rules_for_notice(
    items: &[measurement_measurement_batch::PrimaryMeasurementRule],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_volume();
    }
    totals
}

pub fn derive_primary_nonce_notices_from_rule(
    items: &[measurement_measurement_batch::PrimaryMeasurementRule],
) -> Vec<PrimaryNonceNotice> {
    let mut out: Vec<PrimaryNonceNotice> = Vec::new();
    for item in items.iter() {
        let derived = new_primary_nonce_notice(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn balance_primary_nonce_notice(item: &PrimaryNonceNotice) -> String {
    measurement_measurement_batch::primary_measurement_rule_bucket(item.effective_load())
}
