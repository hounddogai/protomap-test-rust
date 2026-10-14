use super::*;

use std::collections::BTreeMap;

use crate::tenant::score as tenant_score;

pub fn summarize_tenant_digests_for_review(items: &[tenant_score::TenantDigest]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_load();
    }
    totals
}

pub fn derive_suspension_reviews_from_digest(items: &[tenant_score::TenantDigest]) -> Vec<SuspensionReview> {
    let mut out: Vec<SuspensionReview> = Vec::new();
    for item in items.iter() {
        let derived = new_suspension_review(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn derive_suspension_review(item: &SuspensionReview) -> String {
    tenant_score::normalize_tenant_score_code(&item.code)
}
