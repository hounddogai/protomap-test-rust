use super::*;

use std::collections::BTreeMap;

use crate::limit::claim as limit_claim;

pub fn summarize_rolling_limit_claims_for_exception(items: &[limit_claim::RollingLimitClaim]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_overage_exceptions_from_claim(items: &[limit_claim::RollingLimitClaim]) -> Vec<OverageException> {
    let mut out: Vec<OverageException> = Vec::new();
    for item in items.iter() {
        let derived = new_overage_exception(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn classify_overage_exception(item: &OverageException) -> String {
    limit_claim::normalize_limit_group_code(&item.code)
}
