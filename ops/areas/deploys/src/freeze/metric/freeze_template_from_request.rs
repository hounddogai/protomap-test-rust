use super::*;

use std::collections::BTreeMap;

use crate::canary::assignment as canary_assignment;

pub fn summarize_canary_requests_for_template(items: &[canary_assignment::CanaryRequest]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_value();
    }
    totals
}

pub fn derive_freeze_templates_from_request(items: &[canary_assignment::CanaryRequest]) -> Vec<FreezeTemplate> {
    let mut out: Vec<FreezeTemplate> = Vec::new();
    for item in items.iter() {
        let derived = new_freeze_template(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn balance_freeze_template(item: &FreezeTemplate) -> String {
    canary_assignment::normalize_canary_variance_code(&item.code)
}
