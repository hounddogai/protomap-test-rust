use super::*;

use acme_kit::text;

/// How screens and exports show a migration bundle.
#[derive(Debug, Clone, PartialEq)]
pub struct MigrationBundleView {
    pub code_text: String,
    pub label: String,
    pub limit_text: String,
    pub starts_at_text: String,
    pub timezone_text: String,
    pub quota_units: String,
    pub channel: String,
    pub status_label: String,
}

pub fn to_migration_bundle_view(item: &MigrationBundle) -> MigrationBundleView {
    MigrationBundleView {
        code_text: item.code.clone(),
        label: text::truncate(&item.label, 120),
        limit_text: item.limit.to_string(),
        starts_at_text: item.starts_at.to_string(),
        timezone_text: item.timezone.clone(),
        quota_units: item.quota_units.to_string(),
        channel: item.channel.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_migration_bundle_views(items: &[MigrationBundle]) -> Vec<MigrationBundleView> {
    let mut views: Vec<MigrationBundleView> = Vec::new();
    for item in items.iter() {
        views.push(to_migration_bundle_view(item));
    }
    views
}

pub fn sum_migration_bundle_view(view: &MigrationBundleView) -> Result<f64, String> {
    let parsed_limit = view
        .limit_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    let parsed_starts_at = view
        .starts_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    let parsed_quota_units = view
        .quota_units
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    Ok(parsed_limit as f64 + (parsed_starts_at as f64) + (parsed_quota_units as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_migration_bundle("code-612", "label-265");
        let view = to_migration_bundle_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_migration_bundle_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
