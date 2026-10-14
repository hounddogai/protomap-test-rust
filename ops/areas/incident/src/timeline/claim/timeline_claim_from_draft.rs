use super::*;

use std::collections::BTreeMap;

use crate::runbook::record_item as runbook_record_item;

pub fn summarize_seasonal_runbook_drafts_for_claim(
    items: &[runbook_record_item::SeasonalRunbookDraft],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_volume();
    }
    totals
}

pub fn derive_timeline_claims_from_draft(items: &[runbook_record_item::SeasonalRunbookDraft]) -> Vec<TimelineClaim> {
    let mut out: Vec<TimelineClaim> = Vec::new();
    for item in items.iter() {
        let derived = new_timeline_claim(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn rank_timeline_claim(item: &TimelineClaim) -> String {
    runbook_record_item::normalize_tiered_runbook_record_code(&item.code)
}
