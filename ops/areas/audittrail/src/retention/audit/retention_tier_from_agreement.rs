use super::*;

use std::collections::BTreeMap;

use crate::sink::ledger as sink_ledger;

pub fn summarize_global_sink_agreements_for_tier(items: &[sink_ledger::GlobalSinkAgreement]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_exposure();
    }
    totals
}

pub fn derive_retention_tiers_from_agreement(items: &[sink_ledger::GlobalSinkAgreement]) -> Vec<RetentionTier> {
    let mut out: Vec<RetentionTier> = Vec::new();
    for item in items.iter() {
        let derived = new_retention_tier(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn split_retention_tier(item: &RetentionTier) -> String {
    sink_ledger::batched_sink_exception_bucket(item.projected_volume())
}
