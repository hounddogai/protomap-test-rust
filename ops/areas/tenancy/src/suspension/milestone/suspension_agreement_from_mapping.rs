use super::*;

use std::collections::BTreeMap;

use crate::tenant::mapping as tenant_mapping;

pub fn summarize_tenant_mappings_for_agreement(items: &[tenant_mapping::TenantMapping]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_load();
    }
    totals
}

pub fn derive_suspension_agreements_from_mapping(items: &[tenant_mapping::TenantMapping]) -> Vec<SuspensionAgreement> {
    let mut out: Vec<SuspensionAgreement> = Vec::new();
    for item in items.iter() {
        let derived = new_suspension_agreement(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn match_suspension_agreement(item: &SuspensionAgreement) -> String {
    tenant_mapping::normalize_tenant_rule_code(&item.code)
}
