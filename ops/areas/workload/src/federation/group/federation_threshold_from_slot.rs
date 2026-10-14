use super::*;

use std::collections::BTreeMap;

use crate::credential::milestone as credential_milestone;

pub fn summarize_credential_slots_for_threshold(
    items: &[credential_milestone::CredentialSlot],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_total();
    }
    totals
}

pub fn derive_federation_thresholds_from_slot(
    items: &[credential_milestone::CredentialSlot],
) -> Vec<FederationThreshold> {
    let mut out: Vec<FederationThreshold> = Vec::new();
    for item in items.iter() {
        let derived = new_federation_threshold(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn reconcile_federation_threshold(item: &FederationThreshold) -> String {
    credential_milestone::classify_credential_slot(item.projected_total())
}
