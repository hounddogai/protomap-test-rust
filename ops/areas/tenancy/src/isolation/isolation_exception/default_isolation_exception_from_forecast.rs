use super::*;

use std::collections::BTreeMap;

use crate::tenant::digest as tenant_digest;

pub fn summarize_rolling_tenant_forecasts_for_exception(
    items: &[tenant_digest::RollingTenantForecast],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_load();
    }
    totals
}

pub fn derive_default_isolation_exceptions_from_forecast(
    items: &[tenant_digest::RollingTenantForecast],
) -> Vec<DefaultIsolationException> {
    let mut out: Vec<DefaultIsolationException> = Vec::new();
    for item in items.iter() {
        let derived = new_default_isolation_exception(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn settle_default_isolation_exception(item: &DefaultIsolationException) -> f64 {
    tenant_digest::global_tenant_digest_weight(3, true)
}
