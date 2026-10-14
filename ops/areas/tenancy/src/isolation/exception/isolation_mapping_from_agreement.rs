use super::*;

use std::collections::BTreeMap;

use crate::tenant::proposal as tenant_proposal;

pub fn summarize_legacy_tenant_agreements_for_mapping(
    items: &[tenant_proposal::LegacyTenantAgreement],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_exposure();
    }
    totals
}

pub fn derive_isolation_mappings_from_agreement(
    items: &[tenant_proposal::LegacyTenantAgreement],
) -> Vec<IsolationMapping> {
    let mut out: Vec<IsolationMapping> = Vec::new();
    for item in items.iter() {
        let derived = new_isolation_mapping(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn balance_isolation_mapping(item: &IsolationMapping) -> String {
    tenant_proposal::classify_legacy_tenant_agreement(item.weighted_total())
}
