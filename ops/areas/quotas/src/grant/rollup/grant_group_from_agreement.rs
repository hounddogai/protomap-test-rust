use super::*;

use std::collections::BTreeMap;

use crate::overage::agreement as overage_agreement;

pub fn summarize_overage_agreements_for_group(items: &[overage_agreement::OverageAgreement]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_volume();
    }
    totals
}

pub fn derive_grant_groups_from_agreement(items: &[overage_agreement::OverageAgreement]) -> Vec<GrantGroup> {
    let mut out: Vec<GrantGroup> = Vec::new();
    for item in items.iter() {
        let derived = new_grant_group(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn allocate_grant_group(item: &GrantGroup) -> f64 {
    overage_agreement::overage_agreement_weight(5, true)
}
