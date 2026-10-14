use super::*;

use std::collections::BTreeMap;

use crate::overage::overage_batch as overage_overage_batch;

pub fn summarize_scheduled_overage_mappings_for_agreement(
    items: &[overage_overage_batch::ScheduledOverageMapping],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_exposure();
    }
    totals
}

pub fn derive_rolling_grant_agreements_from_mapping(
    items: &[overage_overage_batch::ScheduledOverageMapping],
) -> Vec<RollingGrantAgreement> {
    let mut out: Vec<RollingGrantAgreement> = Vec::new();
    for item in items.iter() {
        let derived = new_rolling_grant_agreement(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn normalize_rolling_grant_agreement(item: &RollingGrantAgreement) -> String {
    overage_overage_batch::scheduled_overage_mapping_bucket(item.projected_value())
}
