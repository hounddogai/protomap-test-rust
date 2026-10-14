use super::*;

use std::collections::BTreeMap;

use crate::page::variance as page_variance;

pub fn summarize_page_audits_for_budget(items: &[page_variance::PageAudit]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_legacy_timeline_budgets_from_audit(items: &[page_variance::PageAudit]) -> Vec<LegacyTimelineBudget> {
    let mut out: Vec<LegacyTimelineBudget> = Vec::new();
    for item in items.iter() {
        let derived = new_legacy_timeline_budget(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn resolve_legacy_timeline_budget(item: &LegacyTimelineBudget) -> String {
    page_variance::page_audit_bucket(item.effective_exposure())
}
