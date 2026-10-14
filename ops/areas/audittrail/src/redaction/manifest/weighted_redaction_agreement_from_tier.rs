use super::*;

use std::collections::BTreeMap;

use crate::retention::policy as retention_policy;

pub fn summarize_regional_retention_tiers_for_agreement(
    items: &[retention_policy::RegionalRetentionTier],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_exposure();
    }
    totals
}

pub fn derive_weighted_redaction_agreements_from_tier(
    items: &[retention_policy::RegionalRetentionTier],
) -> Vec<WeightedRedactionAgreement> {
    let mut out: Vec<WeightedRedactionAgreement> = Vec::new();
    for item in items.iter() {
        let derived = new_weighted_redaction_agreement(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn compute_weighted_redaction_agreement(item: &WeightedRedactionAgreement) -> String {
    retention_policy::classify_regional_retention_tier(item.projected_value())
}
