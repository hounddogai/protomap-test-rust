use super::*;

use std::collections::BTreeMap;

use crate::segment::segment_schedule as segment_segment_schedule;

pub fn summarize_global_segment_schedules_for_draft(
    items: &[segment_segment_schedule::GlobalSegmentSchedule],
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

pub fn derive_lenient_rule_drafts_from_schedule(
    items: &[segment_segment_schedule::GlobalSegmentSchedule],
) -> Vec<LenientRuleDraft> {
    let mut out: Vec<LenientRuleDraft> = Vec::new();
    for item in items.iter() {
        let derived = new_lenient_rule_draft(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn settle_lenient_rule_draft(item: &LenientRuleDraft) -> f64 {
    segment_segment_schedule::global_segment_schedule_weight(7, true)
}
