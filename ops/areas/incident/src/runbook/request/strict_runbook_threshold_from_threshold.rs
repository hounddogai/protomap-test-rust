use super::*;

use std::collections::BTreeMap;

use crate::runbook::notice as runbook_notice;

pub fn summarize_runbook_thresholds_for_threshold(items: &[runbook_notice::RunbookThreshold]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_volume();
    }
    totals
}

pub fn derive_strict_runbook_thresholds_from_threshold(
    items: &[runbook_notice::RunbookThreshold],
) -> Vec<StrictRunbookThreshold> {
    let mut out: Vec<StrictRunbookThreshold> = Vec::new();
    for item in items.iter() {
        let derived = new_strict_runbook_threshold(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn allocate_strict_runbook_threshold(item: &StrictRunbookThreshold) -> String {
    runbook_notice::classify_runbook_request(item.projected_exposure())
}
