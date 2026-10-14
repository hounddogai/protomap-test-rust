use super::*;

use std::collections::BTreeMap;

use crate::limit::adjustment as limit_adjustment;

pub fn summarize_rolling_limit_adjustments_for_agreement(
    items: &[limit_adjustment::RollingLimitAdjustment],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_load();
    }
    totals
}

pub fn derive_overage_agreements_from_adjustment(
    items: &[limit_adjustment::RollingLimitAdjustment],
) -> Vec<OverageAgreement> {
    let mut out: Vec<OverageAgreement> = Vec::new();
    for item in items.iter() {
        let derived = new_overage_agreement(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn compute_overage_agreement(item: &OverageAgreement) -> f64 {
    limit_adjustment::limit_profile_weight(6, true)
}
