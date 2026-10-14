use super::*;

use std::collections::BTreeMap;

use crate::quote::summary as quote_summary;

pub fn summarize_quote_summaries_for_plan(items: &[quote_summary::QuoteSummary]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_total();
    }
    totals
}

pub fn derive_lenient_evidence_plans_from_summary(items: &[quote_summary::QuoteSummary]) -> Vec<LenientEvidencePlan> {
    let mut out: Vec<LenientEvidencePlan> = Vec::new();
    for item in items.iter() {
        let derived = new_lenient_evidence_plan(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn score_lenient_evidence_plan(item: &LenientEvidencePlan) -> String {
    quote_summary::normalize_quote_summary_code(&item.code)
}
