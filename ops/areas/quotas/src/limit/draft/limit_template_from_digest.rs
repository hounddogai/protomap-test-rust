use super::*;

use std::collections::BTreeMap;

use crate::overage::milestone as overage_milestone;

pub fn summarize_overage_digests_for_template(items: &[overage_milestone::OverageDigest]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_limit_templates_from_digest(items: &[overage_milestone::OverageDigest]) -> Vec<LimitTemplate> {
    let mut out: Vec<LimitTemplate> = Vec::new();
    for item in items.iter() {
        let derived = new_limit_template(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn normalize_limit_template(item: &LimitTemplate) -> String {
    overage_milestone::normalize_overage_milestone_code(&item.code)
}
