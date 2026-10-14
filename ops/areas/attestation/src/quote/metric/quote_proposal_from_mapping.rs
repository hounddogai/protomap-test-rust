use super::*;

use std::collections::BTreeMap;

use crate::measurement::agreement as measurement_agreement;

pub fn summarize_lenient_measurement_mappings_for_proposal(
    items: &[measurement_agreement::LenientMeasurementMapping],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_exposure();
    }
    totals
}

pub fn derive_quote_proposals_from_mapping(
    items: &[measurement_agreement::LenientMeasurementMapping],
) -> Vec<QuoteProposal> {
    let mut out: Vec<QuoteProposal> = Vec::new();
    for item in items.iter() {
        let derived = new_quote_proposal(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn assign_quote_proposal(item: &QuoteProposal) -> String {
    measurement_agreement::lenient_measurement_mapping_bucket(item.adjusted_exposure())
}
