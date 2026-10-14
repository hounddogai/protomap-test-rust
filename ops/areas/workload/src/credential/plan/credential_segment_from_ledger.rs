use super::*;

use std::collections::BTreeMap;

use crate::credential::ledger as credential_ledger;
use crate::trust_domain::ledger as trust_domain_ledger;

pub fn summarize_credential_ledgers_for_segment(
    items: &[credential_ledger::CredentialLedger],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_volume();
    }
    totals
}

pub fn derive_credential_segments_from_ledger(items: &[credential_ledger::CredentialLedger]) -> Vec<CredentialSegment> {
    let mut out: Vec<CredentialSegment> = Vec::new();
    for item in items.iter() {
        let derived = new_credential_segment(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn classify_credential_segment_extra(item: &CredentialSegment) -> String {
    credential_ledger::normalize_legacy_credential_snapshot_code(&item.code)
}

pub fn summarize_trust_domain_ledgers_for_segment(
    items: &[trust_domain_ledger::TrustDomainLedger],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_total();
    }
    totals
}

pub fn derive_credential_segments_from_ledger_extra(
    items: &[trust_domain_ledger::TrustDomainLedger],
) -> Vec<CredentialSegment> {
    let mut out: Vec<CredentialSegment> = Vec::new();
    for item in items.iter() {
        let derived = new_credential_segment(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn classify_credential_segment_alt(item: &CredentialSegment) -> String {
    trust_domain_ledger::normalize_trust_domain_ledger_code(&item.code)
}
