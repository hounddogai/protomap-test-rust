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

pub fn derive_plan_mappings_from_agreement(items: &[tenant_proposal::LegacyTenantAgreement]) -> Vec<PlanMapping> {
    let mut out: Vec<PlanMapping> = Vec::new();
    for item in items.iter() {
        let derived = new_plan_mapping(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn allocate_plan_mapping(item: &PlanMapping) -> String {
    tenant_proposal::classify_legacy_tenant_agreement(item.adjusted_volume())
}
