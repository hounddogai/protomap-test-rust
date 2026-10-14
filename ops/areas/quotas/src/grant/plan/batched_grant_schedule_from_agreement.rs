use super::*;

use std::collections::BTreeMap;

use crate::overage::overage_agreement as overage_overage_agreement;

pub fn summarize_strict_overage_agreements_for_schedule(
    items: &[overage_overage_agreement::StrictOverageAgreement],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_total();
    }
    totals
}

pub fn derive_batched_grant_schedules_from_agreement(
    items: &[overage_overage_agreement::StrictOverageAgreement],
) -> Vec<BatchedGrantSchedule> {
    let mut out: Vec<BatchedGrantSchedule> = Vec::new();
    for item in items.iter() {
        let derived = new_batched_grant_schedule(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn rank_batched_grant_schedule(item: &BatchedGrantSchedule) -> String {
    overage_overage_agreement::normalize_regional_overage_entry_code(&item.code)
}
