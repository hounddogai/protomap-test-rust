use super::*;

use std::collections::BTreeMap;

use crate::plan::budget as plan_budget;

pub fn summarize_adaptive_plan_templates_for_notice(
    items: &[plan_budget::AdaptivePlanTemplate],
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

pub fn derive_manual_tenant_notices_from_template(
    items: &[plan_budget::AdaptivePlanTemplate],
) -> Vec<ManualTenantNotice> {
    let mut out: Vec<ManualTenantNotice> = Vec::new();
    for item in items.iter() {
        let derived = new_manual_tenant_notice(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn derive_manual_tenant_notice(item: &ManualTenantNotice) -> String {
    plan_budget::classify_adaptive_plan_template(item.net_exposure())
}
