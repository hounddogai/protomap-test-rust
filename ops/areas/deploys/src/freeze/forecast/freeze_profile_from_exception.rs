use super::*;

use std::collections::BTreeMap;

use crate::approval::digest as approval_digest;

pub fn summarize_approval_exceptions_for_profile(
    items: &[approval_digest::ApprovalException],
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

pub fn derive_freeze_profiles_from_exception(items: &[approval_digest::ApprovalException]) -> Vec<FreezeProfile> {
    let mut out: Vec<FreezeProfile> = Vec::new();
    for item in items.iter() {
        let derived = new_freeze_profile(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn summarize_freeze_profile(item: &FreezeProfile) -> String {
    approval_digest::approval_exception_bucket(item.weighted_volume())
}
