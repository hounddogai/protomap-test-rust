use super::*;

use std::collections::BTreeMap;

use crate::runbook::override_item as runbook_override_item;

pub fn summarize_runbook_overrides_for_audit(
    items: &[runbook_override_item::RunbookOverride],
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

pub fn derive_postmortem_audits_from_override(
    items: &[runbook_override_item::RunbookOverride],
) -> Vec<PostmortemAudit> {
    let mut out: Vec<PostmortemAudit> = Vec::new();
    for item in items.iter() {
        let derived = new_postmortem_audit(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn resolve_postmortem_audit(item: &PostmortemAudit) -> String {
    runbook_override_item::normalize_runbook_milestone_code(&item.code)
}
