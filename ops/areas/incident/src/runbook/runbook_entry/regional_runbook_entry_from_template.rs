use super::*;

use std::collections::BTreeMap;

use crate::runbook::entry as runbook_entry;

pub fn summarize_tiered_runbook_templates_for_entry(
    items: &[runbook_entry::TieredRunbookTemplate],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_value();
    }
    totals
}

pub fn derive_regional_runbook_entries_from_template(
    items: &[runbook_entry::TieredRunbookTemplate],
) -> Vec<RegionalRunbookEntry> {
    let mut out: Vec<RegionalRunbookEntry> = Vec::new();
    for item in items.iter() {
        let derived = new_regional_runbook_entry(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn classify_regional_runbook_entry(item: &RegionalRunbookEntry) -> String {
    runbook_entry::normalize_runbook_entry_code(&item.code)
}
