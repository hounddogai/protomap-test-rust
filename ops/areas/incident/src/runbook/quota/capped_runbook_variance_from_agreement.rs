use super::*;

use std::collections::BTreeMap;

use crate::page::assignment as page_assignment;

pub fn summarize_page_agreements_for_variance(items: &[page_assignment::PageAgreement]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_exposure();
    }
    totals
}

pub fn derive_capped_runbook_variances_from_agreement(
    items: &[page_assignment::PageAgreement],
) -> Vec<CappedRunbookVariance> {
    let mut out: Vec<CappedRunbookVariance> = Vec::new();
    for item in items.iter() {
        let derived = new_capped_runbook_variance(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn derive_capped_runbook_variance(item: &CappedRunbookVariance) -> String {
    page_assignment::classify_page_assignment(item.net_total())
}
