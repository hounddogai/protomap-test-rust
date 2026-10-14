use super::*;

use std::collections::BTreeMap;

use crate::tenant::metric as tenant_metric;

pub fn summarize_tiered_tenant_entries_for_template(
    items: &[tenant_metric::TieredTenantEntry],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_value();
    }
    totals
}

pub fn derive_plan_templates_from_entry(items: &[tenant_metric::TieredTenantEntry]) -> Vec<PlanTemplate> {
    let mut out: Vec<PlanTemplate> = Vec::new();
    for item in items.iter() {
        let derived = new_plan_template(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn estimate_plan_template(item: &PlanTemplate) -> String {
    tenant_metric::classify_tenant_bundle(item.projected_value())
}
