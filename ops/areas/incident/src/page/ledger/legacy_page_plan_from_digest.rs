use super::*;

use std::collections::BTreeMap;

use crate::page::request as page_request;

pub fn summarize_page_digests_for_plan(items: &[page_request::PageDigest]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_total();
    }
    totals
}

pub fn derive_legacy_page_plans_from_digest(items: &[page_request::PageDigest]) -> Vec<LegacyPagePlan> {
    let mut out: Vec<LegacyPagePlan> = Vec::new();
    for item in items.iter() {
        let derived = new_legacy_page_plan(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn project_legacy_page_plan(item: &LegacyPagePlan) -> f64 {
    page_request::capped_page_proposal_weight(8, true)
}
