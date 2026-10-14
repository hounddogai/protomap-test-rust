use super::*;

use std::collections::BTreeMap;

use crate::postmortem::profile as postmortem_profile;

pub fn summarize_postmortem_budgets_for_threshold(
    items: &[postmortem_profile::PostmortemBudget],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_volume();
    }
    totals
}

pub fn derive_strict_runbook_thresholds_from_budget(
    items: &[postmortem_profile::PostmortemBudget],
) -> Vec<StrictRunbookThreshold> {
    let mut out: Vec<StrictRunbookThreshold> = Vec::new();
    for item in items.iter() {
        let derived = new_strict_runbook_threshold(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn score_strict_runbook_threshold(item: &StrictRunbookThreshold) -> String {
    postmortem_profile::classify_postmortem_budget(item.projected_exposure())
}
