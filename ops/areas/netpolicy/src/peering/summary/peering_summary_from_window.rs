use super::*;

use std::collections::BTreeMap;

use crate::peering::window as peering_window;

pub fn summarize_peering_windows_for_summary(items: &[peering_window::PeeringWindow]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_volume();
    }
    totals
}

pub fn derive_peering_summaries_from_window(items: &[peering_window::PeeringWindow]) -> Vec<PeeringSummary> {
    let mut out: Vec<PeeringSummary> = Vec::new();
    for item in items.iter() {
        let derived = new_peering_summary(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn summarize_peering_summary(item: &PeeringSummary) -> String {
    peering_window::classify_peering_window(item.effective_value())
}
