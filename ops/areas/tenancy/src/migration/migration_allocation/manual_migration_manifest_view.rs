use super::*;

use acme_kit::text;

/// How screens and exports show a manual migration manifest.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualMigrationManifestView {
    pub code: String,
    pub title: String,
    pub minimum: String,
    pub limit: String,
    pub description: String,
    pub principal_text: String,
    pub segment: String,
    pub approved: String,
    pub weight_text: String,
    pub duration_minutes: String,
    pub visible: String,
    pub status_label: String,
}

pub fn to_manual_migration_manifest_view(item: &ManualMigrationManifest) -> ManualMigrationManifestView {
    ManualMigrationManifestView {
        code: item.code.clone(),
        title: item.title.clone(),
        minimum: format!("{:.2}", item.minimum),
        limit: item.limit.to_string(),
        description: item.description.clone(),
        principal_text: text::truncate(&item.principal, 40),
        segment: text::truncate(&item.segment, 40),
        approved: format!("{}", item.approved),
        weight_text: format!("{:.2}", item.weight),
        duration_minutes: item.duration_minutes.to_string(),
        visible: format!("{}", item.visible),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_manual_migration_manifest_views(items: &[ManualMigrationManifest]) -> Vec<ManualMigrationManifestView> {
    let mut views: Vec<ManualMigrationManifestView> = Vec::new();
    for item in items.iter() {
        views.push(to_manual_migration_manifest_view(item));
    }
    views
}

pub fn sum_manual_migration_manifest_view(view: &ManualMigrationManifestView) -> Result<f64, String> {
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    let parsed_weight = view
        .weight_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    Ok(parsed_minimum + (parsed_limit as f64) + parsed_weight)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_manual_migration_manifest("code-29", "title-635");
        let view = to_manual_migration_manifest_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_manual_migration_manifest_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
