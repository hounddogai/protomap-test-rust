use super::*;

use std::collections::BTreeMap;

use crate::release::batch as release_batch;

pub fn summarize_release_budgets_for_digest(items: &[release_batch::ReleaseBudget]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_canary_digests_from_budget(items: &[release_batch::ReleaseBudget]) -> Vec<CanaryDigest> {
    let mut out: Vec<CanaryDigest> = Vec::new();
    for item in items.iter() {
        let derived = new_canary_digest(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn derive_canary_digest(item: &CanaryDigest) -> String {
    release_batch::release_digest_bucket(item.measured_value())
}
