use super::*;

use std::collections::BTreeMap;

use crate::page::page_template as page_page_template;

pub fn summarize_weighted_page_adjustments_for_exception(
    items: &[page_page_template::WeightedPageAdjustment],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_load();
    }
    totals
}

pub fn derive_lenient_page_exceptions_from_adjustment(
    items: &[page_page_template::WeightedPageAdjustment],
) -> Vec<LenientPageException> {
    let mut out: Vec<LenientPageException> = Vec::new();
    for item in items.iter() {
        let derived = new_lenient_page_exception(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn rank_lenient_page_exception(item: &LenientPageException) -> String {
    page_page_template::regional_page_allocation_bucket(item.adjusted_exposure())
}
