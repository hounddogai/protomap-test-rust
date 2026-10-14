use super::*;

use acme_kit::text;

/// How screens and exports show a regional migration tier.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalMigrationTierView {
    pub code: String,
    pub title: String,
    pub namespace: String,
    pub source: String,
    pub host: String,
    pub notes: String,
    pub threshold_text: String,
    pub attempts: String,
    pub quantity: String,
    pub checksum: String,
    pub version: String,
    pub status_label: String,
}

pub fn to_regional_migration_tier_view(item: &RegionalMigrationTier) -> RegionalMigrationTierView {
    RegionalMigrationTierView {
        code: item.code.clone(),
        title: item.title.clone(),
        namespace: item.namespace.clone(),
        source: text::truncate(&item.source, 120),
        host: item.host.clone(),
        notes: item.notes.clone(),
        threshold_text: format!("{:.2}", item.threshold),
        attempts: item.attempts.to_string(),
        quantity: item.quantity.to_string(),
        checksum: text::truncate(&item.checksum, 64),
        version: item.version.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_regional_migration_tier_views(items: &[RegionalMigrationTier]) -> Vec<RegionalMigrationTierView> {
    let mut views: Vec<RegionalMigrationTierView> = Vec::new();
    for item in items.iter() {
        views.push(to_regional_migration_tier_view(item));
    }
    views
}

pub fn sum_regional_migration_tier_view(view: &RegionalMigrationTierView) -> Result<f64, String> {
    let parsed_threshold = view
        .threshold_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    let parsed_attempts = view
        .attempts
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    Ok(parsed_threshold + (parsed_attempts as f64) + (parsed_quantity as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_regional_migration_tier("code-412", "title-823");
        let view = to_regional_migration_tier_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_regional_migration_tier_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
