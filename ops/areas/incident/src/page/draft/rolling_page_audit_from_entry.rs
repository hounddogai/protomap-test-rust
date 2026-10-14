use super::*;

use std::collections::BTreeMap;

use crate::page::tier as page_tier;

pub fn summarize_rolling_page_entries_for_audit(items: &[page_tier::RollingPageEntry]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_rolling_page_audits_from_entry(items: &[page_tier::RollingPageEntry]) -> Vec<RollingPageAudit> {
    let mut out: Vec<RollingPageAudit> = Vec::new();
    for item in items.iter() {
        let derived = new_rolling_page_audit(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn classify_rolling_page_audit(item: &RollingPageAudit) -> String {
    page_tier::classify_adaptive_page_proposal(item.projected_volume())
}
