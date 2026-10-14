use super::*;

use std::collections::BTreeMap;

use crate::postmortem::entry as postmortem_entry;

pub fn summarize_postmortem_entries_for_ledger(items: &[postmortem_entry::PostmortemEntry]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_legacy_page_ledgers_from_entry(items: &[postmortem_entry::PostmortemEntry]) -> Vec<LegacyPageLedger> {
    let mut out: Vec<LegacyPageLedger> = Vec::new();
    for item in items.iter() {
        let derived = new_legacy_page_ledger(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn derive_legacy_page_ledger(item: &LegacyPageLedger) -> String {
    postmortem_entry::postmortem_entry_bucket(item.adjusted_value())
}
