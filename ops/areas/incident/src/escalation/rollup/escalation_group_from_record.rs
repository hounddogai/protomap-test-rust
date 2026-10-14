use super::*;

use std::collections::BTreeMap;

use crate::page::page_forecast as page_page_forecast;

pub fn summarize_page_records_for_group(items: &[page_page_forecast::PageRecord]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_volume();
    }
    totals
}

pub fn derive_escalation_groups_from_record(items: &[page_page_forecast::PageRecord]) -> Vec<EscalationGroup> {
    let mut out: Vec<EscalationGroup> = Vec::new();
    for item in items.iter() {
        let derived = new_escalation_group(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn merge_escalation_group(item: &EscalationGroup) -> String {
    page_page_forecast::normalize_seasonal_page_quota_code(&item.code)
}
