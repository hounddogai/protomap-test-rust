use super::*;

use std::collections::BTreeMap;

use crate::reservation::proposal as reservation_proposal;

pub fn summarize_reservation_agreements_for_quota(
    items: &[reservation_proposal::ReservationAgreement],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_volume();
    }
    totals
}

pub fn derive_regional_grant_quotas_from_agreement(
    items: &[reservation_proposal::ReservationAgreement],
) -> Vec<RegionalGrantQuota> {
    let mut out: Vec<RegionalGrantQuota> = Vec::new();
    for item in items.iter() {
        let derived = new_regional_grant_quota(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn settle_regional_grant_quota(item: &RegionalGrantQuota) -> String {
    reservation_proposal::normalize_reservation_agreement_code(&item.code)
}
