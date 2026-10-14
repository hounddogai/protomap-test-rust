use super::*;

use std::collections::BTreeMap;

use crate::runbook::assignment as runbook_assignment;

pub fn summarize_runbook_claims_for_window(items: &[runbook_assignment::RunbookClaim]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_volume();
    }
    totals
}

pub fn derive_runbook_windows_from_claim(items: &[runbook_assignment::RunbookClaim]) -> Vec<RunbookWindow> {
    let mut out: Vec<RunbookWindow> = Vec::new();
    for item in items.iter() {
        let derived = new_runbook_window(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn plan_runbook_window(item: &RunbookWindow) -> String {
    runbook_assignment::normalize_runbook_reservation_code(&item.code)
}
