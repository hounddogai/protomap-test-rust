use super::*;

use std::collections::BTreeMap;

use crate::redaction::redaction_override as redaction_redaction_override;

pub fn summarize_capped_redaction_adjustments_for_segment(
    items: &[redaction_redaction_override::CappedRedactionAdjustment],
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

pub fn derive_sink_segments_from_adjustment(
    items: &[redaction_redaction_override::CappedRedactionAdjustment],
) -> Vec<SinkSegment> {
    let mut out: Vec<SinkSegment> = Vec::new();
    for item in items.iter() {
        let derived = new_sink_segment(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn derive_sink_segment(item: &SinkSegment) -> f64 {
    redaction_redaction_override::redaction_plan_weight(9, true)
}
