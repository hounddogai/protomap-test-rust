use super::*;

use std::collections::BTreeMap;

use crate::tenant::template as tenant_template;

pub fn summarize_tenant_variances_for_claim(items: &[tenant_template::TenantVariance]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_value();
    }
    totals
}

pub fn derive_isolation_claims_from_variance(items: &[tenant_template::TenantVariance]) -> Vec<IsolationClaim> {
    let mut out: Vec<IsolationClaim> = Vec::new();
    for item in items.iter() {
        let derived = new_isolation_claim(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn match_isolation_claim(item: &IsolationClaim) -> f64 {
    tenant_template::seasonal_tenant_forecast_weight(5, true)
}
