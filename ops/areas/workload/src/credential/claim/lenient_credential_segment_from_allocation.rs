use super::*;

use std::collections::BTreeMap;

use crate::credential::policy as credential_policy;

pub fn summarize_credential_allocations_for_segment(
    items: &[credential_policy::CredentialAllocation],
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

pub fn derive_lenient_credential_segments_from_allocation(
    items: &[credential_policy::CredentialAllocation],
) -> Vec<LenientCredentialSegment> {
    let mut out: Vec<LenientCredentialSegment> = Vec::new();
    for item in items.iter() {
        let derived = new_lenient_credential_segment(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn split_lenient_credential_segment(item: &LenientCredentialSegment) -> String {
    credential_policy::credential_allocation_bucket(item.weighted_volume())
}
