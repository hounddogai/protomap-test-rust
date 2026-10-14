use super::*;

use std::collections::BTreeMap;

use crate::trust_domain::audit as trust_domain_audit;

pub fn summarize_trust_domain_audits_for_template(
    items: &[trust_domain_audit::TrustDomainAudit],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_exposure();
    }
    totals
}

pub fn derive_binding_templates_from_audit(items: &[trust_domain_audit::TrustDomainAudit]) -> Vec<BindingTemplate> {
    let mut out: Vec<BindingTemplate> = Vec::new();
    for item in items.iter() {
        let derived = new_binding_template(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn reconcile_binding_template(item: &BindingTemplate) -> f64 {
    trust_domain_audit::trust_domain_template_weight(7, true)
}
