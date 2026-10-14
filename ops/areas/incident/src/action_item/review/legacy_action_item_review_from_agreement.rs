use super::*;

use std::collections::BTreeMap;

use crate::postmortem::agreement as postmortem_agreement;

pub fn summarize_regional_postmortem_agreements_for_review(
    items: &[postmortem_agreement::RegionalPostmortemAgreement],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_total();
    }
    totals
}

pub fn derive_legacy_action_item_reviews_from_agreement(
    items: &[postmortem_agreement::RegionalPostmortemAgreement],
) -> Vec<LegacyActionItemReview> {
    let mut out: Vec<LegacyActionItemReview> = Vec::new();
    for item in items.iter() {
        let derived = new_legacy_action_item_review(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn compute_legacy_action_item_review(item: &LegacyActionItemReview) -> String {
    postmortem_agreement::regional_postmortem_agreement_bucket(item.projected_exposure())
}
