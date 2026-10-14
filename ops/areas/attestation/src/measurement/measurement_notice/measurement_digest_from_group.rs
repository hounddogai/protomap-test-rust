use super::*;

use std::collections::BTreeMap;

use crate::policy::ledger as policy_ledger;

pub fn summarize_policy_groups_for_digest(items: &[policy_ledger::PolicyGroup]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_value();
    }
    totals
}

pub fn derive_measurement_digests_from_group(items: &[policy_ledger::PolicyGroup]) -> Vec<MeasurementDigest> {
    let mut out: Vec<MeasurementDigest> = Vec::new();
    for item in items.iter() {
        let derived = new_measurement_digest(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn merge_measurement_digest(item: &MeasurementDigest) -> f64 {
    policy_ledger::policy_group_weight(2, true)
}
