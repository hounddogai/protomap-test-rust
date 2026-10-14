use super::*;

use std::collections::BTreeMap;

use crate::runbook::agreement as runbook_agreement;

pub fn summarize_runbook_preferences_for_policy(
    items: &[runbook_agreement::RunbookPreference],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_total();
    }
    totals
}

pub fn derive_page_policies_from_preference(items: &[runbook_agreement::RunbookPreference]) -> Vec<PagePolicy> {
    let mut out: Vec<PagePolicy> = Vec::new();
    for item in items.iter() {
        let derived = new_page_policy(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn summarize_page_policy(item: &PagePolicy) -> String {
    runbook_agreement::classify_runbook_preference(item.weighted_exposure())
}
