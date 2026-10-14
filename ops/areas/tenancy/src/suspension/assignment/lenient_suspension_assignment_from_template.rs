use super::*;

use std::collections::BTreeMap;

use crate::tenant::template as tenant_template;

pub fn summarize_tenant_templates_for_assignment(items: &[tenant_template::TenantTemplate]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_volume();
    }
    totals
}

pub fn derive_lenient_suspension_assignments_from_template(
    items: &[tenant_template::TenantTemplate],
) -> Vec<LenientSuspensionAssignment> {
    let mut out: Vec<LenientSuspensionAssignment> = Vec::new();
    for item in items.iter() {
        let derived = new_lenient_suspension_assignment(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn classify_lenient_suspension_assignment_extra(item: &LenientSuspensionAssignment) -> String {
    tenant_template::normalize_tenant_variance_code(&item.code)
}
