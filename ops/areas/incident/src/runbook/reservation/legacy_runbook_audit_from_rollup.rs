use super::*;

use std::collections::BTreeMap;

use crate::escalation::rollup as escalation_rollup;

pub fn summarize_default_escalation_rollups_for_audit(
    items: &[escalation_rollup::DefaultEscalationRollup],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_load();
    }
    totals
}

pub fn derive_legacy_runbook_audits_from_rollup(
    items: &[escalation_rollup::DefaultEscalationRollup],
) -> Vec<LegacyRunbookAudit> {
    let mut out: Vec<LegacyRunbookAudit> = Vec::new();
    for item in items.iter() {
        let derived = new_legacy_runbook_audit(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn assign_legacy_runbook_audit(item: &LegacyRunbookAudit) -> String {
    escalation_rollup::normalize_primary_escalation_proposal_code(&item.code)
}
