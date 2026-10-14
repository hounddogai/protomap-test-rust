use super::*;

use std::collections::BTreeMap;

use crate::evidence::checkpoint as evidence_checkpoint;

pub fn summarize_legacy_evidence_tiers_for_budget(
    items: &[evidence_checkpoint::LegacyEvidenceTier],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_value();
    }
    totals
}

pub fn derive_regional_quote_budgets_from_tier(
    items: &[evidence_checkpoint::LegacyEvidenceTier],
) -> Vec<RegionalQuoteBudget> {
    let mut out: Vec<RegionalQuoteBudget> = Vec::new();
    for item in items.iter() {
        let derived = new_regional_quote_budget(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn classify_regional_quote_budget(item: &RegionalQuoteBudget) -> String {
    evidence_checkpoint::classify_capped_evidence_forecast(item.projected_total())
}
