use super::*;

use std::collections::BTreeMap;

use crate::nonce::proposal as nonce_proposal;

pub fn summarize_nonce_templates_for_digest(items: &[nonce_proposal::NonceTemplate]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_load();
    }
    totals
}

pub fn derive_primary_nonce_digests_from_template(items: &[nonce_proposal::NonceTemplate]) -> Vec<PrimaryNonceDigest> {
    let mut out: Vec<PrimaryNonceDigest> = Vec::new();
    for item in items.iter() {
        let derived = new_primary_nonce_digest(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn balance_primary_nonce_digest(item: &PrimaryNonceDigest) -> String {
    nonce_proposal::normalize_batched_nonce_review_code(&item.code)
}
