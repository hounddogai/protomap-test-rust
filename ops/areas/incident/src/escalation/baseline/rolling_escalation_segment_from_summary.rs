use super::*;

use std::collections::BTreeMap;

use crate::page::allocation as page_allocation;

pub fn summarize_capped_page_summaries_for_segment(
    items: &[page_allocation::CappedPageSummary],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_total();
    }
    totals
}

pub fn derive_rolling_escalation_segments_from_summary(
    items: &[page_allocation::CappedPageSummary],
) -> Vec<RollingEscalationSegment> {
    let mut out: Vec<RollingEscalationSegment> = Vec::new();
    for item in items.iter() {
        let derived = new_rolling_escalation_segment(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn score_rolling_escalation_segment(item: &RollingEscalationSegment) -> String {
    page_allocation::normalize_page_allocation_code(&item.code)
}
