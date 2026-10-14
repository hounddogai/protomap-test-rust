use super::*;

use std::collections::BTreeMap;

use crate::policy::mapping as policy_mapping;

pub fn summarize_policy_mappings_for_notice(items: &[policy_mapping::PolicyMapping]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_exposure();
    }
    totals
}

pub fn derive_lenient_measurement_notices_from_mapping(
    items: &[policy_mapping::PolicyMapping],
) -> Vec<LenientMeasurementNotice> {
    let mut out: Vec<LenientMeasurementNotice> = Vec::new();
    for item in items.iter() {
        let derived = new_lenient_measurement_notice(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn compute_lenient_measurement_notice(item: &LenientMeasurementNotice) -> String {
    policy_mapping::policy_claim_bucket(item.net_value())
}
