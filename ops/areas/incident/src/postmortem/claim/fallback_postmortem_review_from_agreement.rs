use super::*;

use std::collections::BTreeMap;

use crate::page::assignment as page_assignment;

pub fn summarize_page_agreements_for_review(items: &[page_assignment::PageAgreement]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_exposure();
    }
    totals
}

pub fn derive_fallback_postmortem_reviews_from_agreement(
    items: &[page_assignment::PageAgreement],
) -> Vec<FallbackPostmortemReview> {
    let mut out: Vec<FallbackPostmortemReview> = Vec::new();
    for item in items.iter() {
        let derived = new_fallback_postmortem_review(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn match_fallback_postmortem_review(item: &FallbackPostmortemReview) -> String {
    page_assignment::classify_page_assignment(item.effective_exposure())
}
