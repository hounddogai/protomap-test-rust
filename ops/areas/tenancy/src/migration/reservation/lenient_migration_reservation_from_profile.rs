use super::*;

use std::collections::BTreeMap;

use crate::migration::profile as migration_profile;

pub fn summarize_migration_profiles_for_reservation(
    items: &[migration_profile::MigrationProfile],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_volume();
    }
    totals
}

pub fn derive_lenient_migration_reservations_from_profile(
    items: &[migration_profile::MigrationProfile],
) -> Vec<LenientMigrationReservation> {
    let mut out: Vec<LenientMigrationReservation> = Vec::new();
    for item in items.iter() {
        let derived = new_lenient_migration_reservation(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn reconcile_lenient_migration_reservation(item: &LenientMigrationReservation) -> String {
    migration_profile::normalize_migration_profile_code(&item.code)
}
