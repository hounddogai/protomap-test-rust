use super::*;

use std::collections::BTreeMap;

use crate::timeline::agreement as timeline_agreement;

pub fn summarize_timeline_agreements_for_agreement(
    items: &[timeline_agreement::TimelineAgreement],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_value();
    }
    totals
}

pub fn derive_weighted_runbook_agreements_from_agreement(
    items: &[timeline_agreement::TimelineAgreement],
) -> Vec<WeightedRunbookAgreement> {
    let mut out: Vec<WeightedRunbookAgreement> = Vec::new();
    for item in items.iter() {
        let derived = new_weighted_runbook_agreement(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn match_weighted_runbook_agreement(item: &WeightedRunbookAgreement) -> String {
    timeline_agreement::classify_timeline_agreement(item.net_value())
}
