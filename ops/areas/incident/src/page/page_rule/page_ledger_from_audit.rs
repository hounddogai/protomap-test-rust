use super::*;

use std::collections::BTreeMap;

use crate::postmortem::rollup as postmortem_rollup;

pub fn summarize_postmortem_audits_for_ledger(items: &[postmortem_rollup::PostmortemAudit]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_page_ledgers_from_audit(items: &[postmortem_rollup::PostmortemAudit]) -> Vec<PageLedger> {
    let mut out: Vec<PageLedger> = Vec::new();
    for item in items.iter() {
        let derived = new_page_ledger(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn assign_page_ledger(item: &PageLedger) -> String {
    postmortem_rollup::classify_postmortem_rollup(item.net_exposure())
}
