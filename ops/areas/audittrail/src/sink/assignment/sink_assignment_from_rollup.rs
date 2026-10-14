use super::*;

use std::collections::BTreeMap;

use crate::redaction::score as redaction_score;

pub fn summarize_legacy_redaction_rollups_for_assignment(
    items: &[redaction_score::LegacyRedactionRollup],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_load();
    }
    totals
}

pub fn derive_sink_assignments_from_rollup(items: &[redaction_score::LegacyRedactionRollup]) -> Vec<SinkAssignment> {
    let mut out: Vec<SinkAssignment> = Vec::new();
    for item in items.iter() {
        let derived = new_sink_assignment(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn match_sink_assignment(item: &SinkAssignment) -> String {
    redaction_score::normalize_rolling_redaction_score_code(&item.code)
}
