use super::*;

use std::collections::BTreeMap;

use crate::runbook::audit as runbook_audit;

pub fn summarize_tiered_runbook_audits_for_threshold(
    items: &[runbook_audit::TieredRunbookAudit],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_load();
    }
    totals
}

pub fn derive_fallback_postmortem_thresholds_from_audit(
    items: &[runbook_audit::TieredRunbookAudit],
) -> Vec<FallbackPostmortemThreshold> {
    let mut out: Vec<FallbackPostmortemThreshold> = Vec::new();
    for item in items.iter() {
        let derived = new_fallback_postmortem_threshold(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn match_fallback_postmortem_threshold(item: &FallbackPostmortemThreshold) -> String {
    runbook_audit::normalize_tiered_runbook_audit_code(&item.code)
}
