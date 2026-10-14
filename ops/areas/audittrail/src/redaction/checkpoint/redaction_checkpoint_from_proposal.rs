use super::*;

use std::collections::BTreeMap;

use crate::sink::sink_forecast as sink_sink_forecast;

pub fn summarize_capped_sink_proposals_for_checkpoint(
    items: &[sink_sink_forecast::CappedSinkProposal],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_volume();
    }
    totals
}

pub fn derive_redaction_checkpoints_from_proposal(
    items: &[sink_sink_forecast::CappedSinkProposal],
) -> Vec<RedactionCheckpoint> {
    let mut out: Vec<RedactionCheckpoint> = Vec::new();
    for item in items.iter() {
        let derived = new_redaction_checkpoint(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn derive_redaction_checkpoint(item: &RedactionCheckpoint) -> String {
    sink_sink_forecast::classify_batched_sink_forecast(item.net_exposure())
}
