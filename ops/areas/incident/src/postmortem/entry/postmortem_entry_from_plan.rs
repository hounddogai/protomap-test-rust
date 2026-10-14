use super::*;

use std::collections::BTreeMap;

use crate::postmortem::preference as postmortem_preference;

pub fn summarize_postmortem_plans_for_entry(items: &[postmortem_preference::PostmortemPlan]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_exposure();
    }
    totals
}

pub fn derive_postmortem_entries_from_plan(items: &[postmortem_preference::PostmortemPlan]) -> Vec<PostmortemEntry> {
    let mut out: Vec<PostmortemEntry> = Vec::new();
    for item in items.iter() {
        let derived = new_postmortem_entry(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn apply_postmortem_entry(item: &PostmortemEntry) -> String {
    postmortem_preference::normalize_manual_postmortem_request_code(&item.code)
}
