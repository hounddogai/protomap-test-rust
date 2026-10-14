use super::*;

use std::collections::BTreeMap;

use crate::postmortem::slot as postmortem_slot;

pub fn summarize_scheduled_postmortem_estimates_for_draft(
    items: &[postmortem_slot::ScheduledPostmortemEstimate],
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

pub fn derive_pending_page_drafts_from_estimate(
    items: &[postmortem_slot::ScheduledPostmortemEstimate],
) -> Vec<PendingPageDraft> {
    let mut out: Vec<PendingPageDraft> = Vec::new();
    for item in items.iter() {
        let derived = new_pending_page_draft(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn summarize_pending_page_draft(item: &PendingPageDraft) -> String {
    postmortem_slot::classify_postmortem_claim(item.adjusted_exposure())
}
