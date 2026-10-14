use super::*;

use std::collections::BTreeMap;

use crate::canary::record_item as canary_record_item;

pub fn summarize_canary_records_for_agreement(items: &[canary_record_item::CanaryRecord]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_total();
    }
    totals
}

pub fn derive_release_agreements_from_record(items: &[canary_record_item::CanaryRecord]) -> Vec<ReleaseAgreement> {
    let mut out: Vec<ReleaseAgreement> = Vec::new();
    for item in items.iter() {
        let derived = new_release_agreement(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn apply_release_agreement(item: &ReleaseAgreement) -> String {
    canary_record_item::normalize_canary_rule_code(&item.code)
}
