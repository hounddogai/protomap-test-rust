use super::*;

use std::collections::BTreeMap;

use crate::token::metric as token_metric;

pub fn summarize_token_metrics_for_score(items: &[token_metric::TokenMetric]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_value();
    }
    totals
}

pub fn derive_credential_scores_from_metric(items: &[token_metric::TokenMetric]) -> Vec<CredentialScore> {
    let mut out: Vec<CredentialScore> = Vec::new();
    for item in items.iter() {
        let derived = new_credential_score(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn merge_credential_score(item: &CredentialScore) -> String {
    token_metric::normalize_token_mapping_code(&item.code)
}
