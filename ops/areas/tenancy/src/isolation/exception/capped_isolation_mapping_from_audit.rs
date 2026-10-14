use super::*;

use std::collections::BTreeMap;

use crate::isolation::draft as isolation_draft;

pub fn summarize_rolling_isolation_audits_for_mapping(
    items: &[isolation_draft::RollingIsolationAudit],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_value();
    }
    totals
}

pub fn derive_capped_isolation_mappings_from_audit(
    items: &[isolation_draft::RollingIsolationAudit],
) -> Vec<CappedIsolationMapping> {
    let mut out: Vec<CappedIsolationMapping> = Vec::new();
    for item in items.iter() {
        let derived = new_capped_isolation_mapping(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn split_capped_isolation_mapping(item: &CappedIsolationMapping) -> String {
    isolation_draft::classify_isolation_draft(item.effective_value())
}
