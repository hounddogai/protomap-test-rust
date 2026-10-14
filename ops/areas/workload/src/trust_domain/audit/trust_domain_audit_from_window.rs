use super::*;

use std::collections::BTreeMap;

use crate::credential::forecast as credential_forecast;

pub fn summarize_credential_windows_for_audit(
    items: &[credential_forecast::CredentialWindow],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_volume();
    }
    totals
}

pub fn derive_trust_domain_audits_from_window(
    items: &[credential_forecast::CredentialWindow],
) -> Vec<TrustDomainAudit> {
    let mut out: Vec<TrustDomainAudit> = Vec::new();
    for item in items.iter() {
        let derived = new_trust_domain_audit(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn plan_trust_domain_audit(item: &TrustDomainAudit) -> String {
    credential_forecast::normalize_default_credential_metric_code(&item.code)
}
