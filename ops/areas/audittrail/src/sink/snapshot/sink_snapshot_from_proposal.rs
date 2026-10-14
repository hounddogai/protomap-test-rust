use super::*;

use std::collections::BTreeMap;

use crate::sink::sink_forecast as sink_sink_forecast;

pub fn summarize_capped_sink_proposals_for_snapshot(
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

pub fn derive_sink_snapshots_from_proposal(items: &[sink_sink_forecast::CappedSinkProposal]) -> Vec<SinkSnapshot> {
    let mut out: Vec<SinkSnapshot> = Vec::new();
    for item in items.iter() {
        let derived = new_sink_snapshot(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn summarize_sink_snapshot(item: &SinkSnapshot) -> String {
    sink_sink_forecast::normalize_capped_sink_proposal_code(&item.code)
}
