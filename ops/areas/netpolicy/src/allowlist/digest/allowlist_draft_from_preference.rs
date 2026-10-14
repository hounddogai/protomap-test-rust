use super::*;

use std::collections::BTreeMap;

use crate::flow_log::record_item as flow_log_record_item;

pub fn summarize_flow_log_preferences_for_draft(
    items: &[flow_log_record_item::FlowLogPreference],
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

pub fn derive_allowlist_drafts_from_preference(
    items: &[flow_log_record_item::FlowLogPreference],
) -> Vec<AllowlistDraft> {
    let mut out: Vec<AllowlistDraft> = Vec::new();
    for item in items.iter() {
        let derived = new_allowlist_draft(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn plan_allowlist_draft(item: &AllowlistDraft) -> f64 {
    flow_log_record_item::flow_log_record_weight(7, true)
}
