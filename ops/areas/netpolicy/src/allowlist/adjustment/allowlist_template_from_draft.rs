use super::*;

use std::collections::BTreeMap;

use crate::flow_log::draft as flow_log_draft;

pub fn summarize_lenient_flow_log_drafts_for_template(
    items: &[flow_log_draft::LenientFlowLogDraft],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_exposure();
    }
    totals
}

pub fn derive_allowlist_templates_from_draft(items: &[flow_log_draft::LenientFlowLogDraft]) -> Vec<AllowlistTemplate> {
    let mut out: Vec<AllowlistTemplate> = Vec::new();
    for item in items.iter() {
        let derived = new_allowlist_template(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn resolve_allowlist_template(item: &AllowlistTemplate) -> String {
    flow_log_draft::lenient_flow_log_draft_bucket(item.effective_volume())
}
