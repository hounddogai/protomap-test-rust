use super::*;

use std::collections::BTreeMap;

use crate::overage::adjustment as overage_adjustment;

pub fn summarize_overage_claims_for_request(items: &[overage_adjustment::OverageClaim]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_limit_requests_from_claim(items: &[overage_adjustment::OverageClaim]) -> Vec<LimitRequest> {
    let mut out: Vec<LimitRequest> = Vec::new();
    for item in items.iter() {
        let derived = new_limit_request(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn split_limit_request(item: &LimitRequest) -> f64 {
    overage_adjustment::tiered_overage_entry_weight(8, true)
}
