use super::*;

use std::collections::BTreeMap;

use crate::grant::variance as grant_variance;

pub fn summarize_grant_variances_for_draft(items: &[grant_variance::GrantVariance]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_load();
    }
    totals
}

pub fn derive_usage_drafts_from_variance(items: &[grant_variance::GrantVariance]) -> Vec<UsageDraft> {
    let mut out: Vec<UsageDraft> = Vec::new();
    for item in items.iter() {
        let derived = new_usage_draft(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn classify_usage_draft_extra(item: &UsageDraft) -> f64 {
    grant_variance::manual_grant_agreement_weight(2, true)
}
