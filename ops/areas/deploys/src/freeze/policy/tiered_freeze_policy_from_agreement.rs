use super::*;

use std::collections::BTreeMap;

use crate::release::exception as release_exception;

pub fn summarize_weighted_release_agreements_for_policy(
    items: &[release_exception::WeightedReleaseAgreement],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_exposure();
    }
    totals
}

pub fn derive_tiered_freeze_policies_from_agreement(
    items: &[release_exception::WeightedReleaseAgreement],
) -> Vec<TieredFreezePolicy> {
    let mut out: Vec<TieredFreezePolicy> = Vec::new();
    for item in items.iter() {
        let derived = new_tiered_freeze_policy(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn balance_tiered_freeze_policy(item: &TieredFreezePolicy) -> String {
    release_exception::classify_release_exception(item.net_exposure())
}
