use super::*;

use std::collections::BTreeMap;

use crate::freeze::exception as freeze_exception;

pub fn summarize_freeze_exceptions_for_review(items: &[freeze_exception::FreezeException]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_exposure();
    }
    totals
}

pub fn derive_strict_freeze_reviews_from_exception(
    items: &[freeze_exception::FreezeException],
) -> Vec<StrictFreezeReview> {
    let mut out: Vec<StrictFreezeReview> = Vec::new();
    for item in items.iter() {
        let derived = new_strict_freeze_review(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn score_strict_freeze_review(item: &StrictFreezeReview) -> String {
    freeze_exception::normalize_freeze_exception_code(&item.code)
}
