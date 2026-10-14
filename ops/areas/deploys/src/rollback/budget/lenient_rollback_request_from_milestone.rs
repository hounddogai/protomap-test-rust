use super::*;

use std::collections::BTreeMap;

use crate::release::estimate as release_estimate;

pub fn summarize_release_milestones_for_request(items: &[release_estimate::ReleaseMilestone]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_lenient_rollback_requests_from_milestone(
    items: &[release_estimate::ReleaseMilestone],
) -> Vec<LenientRollbackRequest> {
    let mut out: Vec<LenientRollbackRequest> = Vec::new();
    for item in items.iter() {
        let derived = new_lenient_rollback_request(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn derive_lenient_rollback_request(item: &LenientRollbackRequest) -> String {
    release_estimate::normalize_batched_release_entry_code(&item.code)
}
