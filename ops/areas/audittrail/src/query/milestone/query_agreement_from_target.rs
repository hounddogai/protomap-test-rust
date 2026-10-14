use super::*;

use std::collections::BTreeMap;

use crate::sink::targets as sink_targets;

pub fn summarize_manual_sink_targets_for_agreement(items: &[sink_targets::ManualSinkTarget]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_exposure();
    }
    totals
}

pub fn derive_query_agreements_from_target(items: &[sink_targets::ManualSinkTarget]) -> Vec<QueryAgreement> {
    let mut out: Vec<QueryAgreement> = Vec::new();
    for item in items.iter() {
        let derived = new_query_agreement(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn reconcile_query_agreement(item: &QueryAgreement) -> String {
    sink_targets::normalize_manual_sink_target_code(&item.code)
}
