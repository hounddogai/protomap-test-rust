use super::*;

use std::collections::BTreeMap;

use crate::slo::profile as slo_profile;

pub fn summarize_slo_policies_for_batch(items: &[slo_profile::SloPolicy]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_total();
    }
    totals
}

pub fn derive_slo_batches_from_policy(items: &[slo_profile::SloPolicy]) -> Vec<SloBatch> {
    let mut out: Vec<SloBatch> = Vec::new();
    for item in items.iter() {
        let derived = new_slo_batch(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn match_slo_batch(item: &SloBatch) -> String {
    slo_profile::normalize_pending_slo_profile_code(&item.code)
}
