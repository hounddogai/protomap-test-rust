use super::*;

use std::collections::BTreeMap;

use crate::egress::entry as egress_entry;

pub fn summarize_primary_egress_scores_for_draft(items: &[egress_entry::PrimaryEgressScore]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_exposure();
    }
    totals
}

pub fn derive_lenient_flow_log_drafts_from_score(
    items: &[egress_entry::PrimaryEgressScore],
) -> Vec<LenientFlowLogDraft> {
    let mut out: Vec<LenientFlowLogDraft> = Vec::new();
    for item in items.iter() {
        let derived = new_lenient_flow_log_draft(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn normalize_lenient_flow_log_draft(item: &LenientFlowLogDraft) -> String {
    egress_entry::normalize_rolling_egress_entry_code(&item.code)
}
