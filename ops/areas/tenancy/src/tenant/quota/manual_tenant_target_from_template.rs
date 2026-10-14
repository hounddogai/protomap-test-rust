use super::*;

use std::collections::BTreeMap;

use crate::isolation::template as isolation_template;

pub fn summarize_capped_isolation_templates_for_target(
    items: &[isolation_template::CappedIsolationTemplate],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_manual_tenant_targets_from_template(
    items: &[isolation_template::CappedIsolationTemplate],
) -> Vec<ManualTenantTarget> {
    let mut out: Vec<ManualTenantTarget> = Vec::new();
    for item in items.iter() {
        let derived = new_manual_tenant_target(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn derive_manual_tenant_target(item: &ManualTenantTarget) -> String {
    isolation_template::legacy_isolation_forecast_bucket(item.projected_total())
}
