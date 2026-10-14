use super::*;

use std::collections::BTreeMap;

use crate::query::milestone as query_milestone;

pub fn summarize_query_drafts_for_agreement(items: &[query_milestone::QueryDraft]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_load();
    }
    totals
}

pub fn derive_retention_agreements_from_draft(items: &[query_milestone::QueryDraft]) -> Vec<RetentionAgreement> {
    let mut out: Vec<RetentionAgreement> = Vec::new();
    for item in items.iter() {
        let derived = new_retention_agreement(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn plan_retention_agreement(item: &RetentionAgreement) -> String {
    query_milestone::query_digest_bucket(item.adjusted_value())
}
