use super::*;

use std::collections::BTreeMap;

use crate::migration::adjustment as migration_adjustment;

pub fn summarize_weighted_migration_claims_for_profile(
    items: &[migration_adjustment::WeightedMigrationClaim],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_value();
    }
    totals
}

pub fn derive_suspension_profiles_from_claim(
    items: &[migration_adjustment::WeightedMigrationClaim],
) -> Vec<SuspensionProfile> {
    let mut out: Vec<SuspensionProfile> = Vec::new();
    for item in items.iter() {
        let derived = new_suspension_profile(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn project_suspension_profile(item: &SuspensionProfile) -> String {
    migration_adjustment::classify_regional_migration_adjustment(item.measured_value())
}
