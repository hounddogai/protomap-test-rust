use super::*;

use std::collections::BTreeMap;

use crate::sink::sink_plan as sink_sink_plan;

pub fn summarize_scheduled_sink_plans_for_draft(items: &[sink_sink_plan::ScheduledSinkPlan]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_volume();
    }
    totals
}

pub fn derive_batched_redaction_drafts_from_plan(
    items: &[sink_sink_plan::ScheduledSinkPlan],
) -> Vec<BatchedRedactionDraft> {
    let mut out: Vec<BatchedRedactionDraft> = Vec::new();
    for item in items.iter() {
        let derived = new_batched_redaction_draft(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn settle_batched_redaction_draft(item: &BatchedRedactionDraft) -> f64 {
    sink_sink_plan::primary_sink_plan_weight(8, true)
}
