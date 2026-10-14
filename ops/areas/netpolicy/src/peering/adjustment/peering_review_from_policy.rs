use super::*;

use std::collections::BTreeMap;

use crate::peering::request as peering_request;

pub fn summarize_peering_policies_for_review(items: &[peering_request::PeeringPolicy]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_load();
    }
    totals
}

pub fn derive_peering_reviews_from_policy(items: &[peering_request::PeeringPolicy]) -> Vec<PeeringReview> {
    let mut out: Vec<PeeringReview> = Vec::new();
    for item in items.iter() {
        let derived = new_peering_review(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn summarize_peering_review(item: &PeeringReview) -> String {
    peering_request::classify_peering_variance(item.net_volume())
}
