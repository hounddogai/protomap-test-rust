use super::*;

use std::collections::BTreeMap;

use crate::suspension::request as suspension_request;

pub fn summarize_suspension_baselines_for_assignment(
    items: &[suspension_request::SuspensionBaseline],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_lenient_suspension_assignments_from_baseline(
    items: &[suspension_request::SuspensionBaseline],
) -> Vec<LenientSuspensionAssignment> {
    let mut out: Vec<LenientSuspensionAssignment> = Vec::new();
    for item in items.iter() {
        let derived = new_lenient_suspension_assignment(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn score_lenient_suspension_assignment(item: &LenientSuspensionAssignment) -> String {
    suspension_request::classify_regional_suspension_budget(item.weighted_volume())
}
