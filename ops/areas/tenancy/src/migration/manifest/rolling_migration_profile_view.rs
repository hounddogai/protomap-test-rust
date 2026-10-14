use super::*;

use acme_kit::text;

/// How screens and exports show a rolling migration profile.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingMigrationProfileView {
    pub code_text: String,
    pub name: String,
    pub version: String,
    pub created_at_text: String,
    pub floor_value: String,
    pub principal: String,
    pub quota_units: String,
    pub source_text: String,
    pub ceiling: String,
    pub key_id: String,
    pub owner_id: String,
    pub status_label: String,
}

pub fn to_rolling_migration_profile_view(item: &RollingMigrationProfile) -> RollingMigrationProfileView {
    RollingMigrationProfileView {
        code_text: item.code.clone(),
        name: item.name.clone(),
        version: item.version.to_string(),
        created_at_text: item.created_at.to_string(),
        floor_value: format!("{:.2}", item.floor_value),
        principal: item.principal.clone(),
        quota_units: item.quota_units.to_string(),
        source_text: item.source.clone(),
        ceiling: format!("{:.2}", item.ceiling),
        key_id: item.key_id.clone(),
        owner_id: text::truncate(&item.owner_id, 80),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rolling_migration_profile_views(items: &[RollingMigrationProfile]) -> Vec<RollingMigrationProfileView> {
    let mut views: Vec<RollingMigrationProfileView> = Vec::new();
    for item in items.iter() {
        views.push(to_rolling_migration_profile_view(item));
    }
    views
}

pub fn sum_rolling_migration_profile_view(view: &RollingMigrationProfileView) -> Result<f64, String> {
    let parsed_version = view
        .version
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    let parsed_created_at = view
        .created_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    let parsed_floor_value = view
        .floor_value
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    Ok(parsed_version as f64 + (parsed_created_at as f64) + parsed_floor_value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_rolling_migration_profile("code-210", "name-199");
        let view = to_rolling_migration_profile_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_rolling_migration_profile_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
