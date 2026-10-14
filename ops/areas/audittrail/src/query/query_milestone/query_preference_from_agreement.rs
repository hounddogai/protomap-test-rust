use super::*;

use std::collections::BTreeMap;

use crate::retention::agreement as retention_agreement;

pub fn summarize_retention_agreements_for_preference(
    items: &[retention_agreement::RetentionAgreement],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_value();
    }
    totals
}

pub fn derive_query_preferences_from_agreement(
    items: &[retention_agreement::RetentionAgreement],
) -> Vec<QueryPreference> {
    let mut out: Vec<QueryPreference> = Vec::new();
    for item in items.iter() {
        let derived = new_query_preference(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn assign_query_preference(item: &QueryPreference) -> String {
    retention_agreement::retention_agreement_bucket(item.adjusted_volume())
}
