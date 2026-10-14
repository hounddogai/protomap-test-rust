use super::*;

use std::collections::BTreeMap;

use crate::trust_domain::adjustment as trust_domain_adjustment;

pub fn summarize_trust_domain_adjustments_for_window(
    items: &[trust_domain_adjustment::TrustDomainAdjustment],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_volume();
    }
    totals
}

pub fn derive_credential_windows_from_adjustment(
    items: &[trust_domain_adjustment::TrustDomainAdjustment],
) -> Vec<CredentialWindow> {
    let mut out: Vec<CredentialWindow> = Vec::new();
    for item in items.iter() {
        let derived = new_credential_window(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn reconcile_credential_window(item: &CredentialWindow) -> f64 {
    trust_domain_adjustment::trust_domain_adjustment_weight(5, true)
}
