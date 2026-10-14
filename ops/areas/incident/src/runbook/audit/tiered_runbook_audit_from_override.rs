use super::*;

use std::collections::BTreeMap;

use crate::postmortem::override_item as postmortem_override_item;

pub fn summarize_pending_postmortem_overrides_for_audit(
    items: &[postmortem_override_item::PendingPostmortemOverride],
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

pub fn derive_tiered_runbook_audits_from_override(
    items: &[postmortem_override_item::PendingPostmortemOverride],
) -> Vec<TieredRunbookAudit> {
    let mut out: Vec<TieredRunbookAudit> = Vec::new();
    for item in items.iter() {
        let derived = new_tiered_runbook_audit(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn assign_tiered_runbook_audit(item: &TieredRunbookAudit) -> String {
    postmortem_override_item::normalize_rolling_postmortem_score_code(&item.code)
}
