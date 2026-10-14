use super::*;

use std::collections::BTreeMap;

use crate::identity::template as identity_template;

pub fn summarize_identity_templates_for_metric(items: &[identity_template::IdentityTemplate]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_default_credential_metrics_from_template(
    items: &[identity_template::IdentityTemplate],
) -> Vec<DefaultCredentialMetric> {
    let mut out: Vec<DefaultCredentialMetric> = Vec::new();
    for item in items.iter() {
        let derived = new_default_credential_metric(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn reconcile_default_credential_metric(item: &DefaultCredentialMetric) -> String {
    identity_template::normalize_identity_template_code(&item.code)
}
