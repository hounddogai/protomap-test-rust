use super::*;

use std::collections::BTreeMap;

use crate::flow_log::forecast as flow_log_forecast;

pub fn summarize_flow_log_drafts_for_estimate(items: &[flow_log_forecast::FlowLogDraft]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_volume();
    }
    totals
}

pub fn derive_lenient_allowlist_estimates_from_draft(
    items: &[flow_log_forecast::FlowLogDraft],
) -> Vec<LenientAllowlistEstimate> {
    let mut out: Vec<LenientAllowlistEstimate> = Vec::new();
    for item in items.iter() {
        let derived = new_lenient_allowlist_estimate(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn settle_lenient_allowlist_estimate(item: &LenientAllowlistEstimate) -> String {
    flow_log_forecast::classify_flow_log_draft(item.effective_volume())
}
