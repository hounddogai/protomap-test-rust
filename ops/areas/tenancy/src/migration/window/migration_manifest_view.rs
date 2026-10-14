use super::*;

use acme_kit::text;

/// How screens and exports show a migration manifest.
#[derive(Debug, Clone, PartialEq)]
pub struct MigrationManifestView {
    pub code: String,
    pub label: String,
    pub scope: String,
    pub starts_at: String,
    pub margin: String,
    pub key_id: String,
    pub status_label: String,
}

pub fn to_migration_manifest_view(item: &MigrationManifest) -> MigrationManifestView {
    MigrationManifestView {
        code: item.code.clone(),
        label: text::truncate(&item.label, 40),
        scope: text::truncate(&item.scope, 40),
        starts_at: item.starts_at.to_string(),
        margin: format!("{:.2}", item.margin),
        key_id: item.key_id.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_migration_manifest_views(items: &[MigrationManifest]) -> Vec<MigrationManifestView> {
    let mut views: Vec<MigrationManifestView> = Vec::new();
    for item in items.iter() {
        views.push(to_migration_manifest_view(item));
    }
    views
}

pub fn sum_migration_manifest_view(view: &MigrationManifestView) -> Result<f64, String> {
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    Ok(parsed_starts_at as f64 + parsed_margin)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_migration_manifest("code-268", "label-126");
        let view = to_migration_manifest_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_migration_manifest_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
