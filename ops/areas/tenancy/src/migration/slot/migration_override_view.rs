use super::*;

/// How screens and exports show a migration override.
#[derive(Debug, Clone, PartialEq)]
pub struct MigrationOverrideView {
    pub code_text: String,
    pub title: String,
    pub scope: String,
    pub pinned: String,
    pub duration_minutes: String,
    pub region: String,
    pub status_label: String,
}

pub fn to_migration_override_view(item: &MigrationOverride) -> MigrationOverrideView {
    MigrationOverrideView {
        code_text: item.code.clone(),
        title: item.title.clone(),
        scope: item.scope.clone(),
        pinned: format!("{}", item.pinned),
        duration_minutes: item.duration_minutes.to_string(),
        region: item.region.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_migration_override_views(items: &[MigrationOverride]) -> Vec<MigrationOverrideView> {
    let mut views: Vec<MigrationOverrideView> = Vec::new();
    for item in items.iter() {
        views.push(to_migration_override_view(item));
    }
    views
}

pub fn sum_migration_override_view(view: &MigrationOverrideView) -> Result<f64, String> {
    let parsed_duration_minutes = view
        .duration_minutes
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    Ok(parsed_duration_minutes as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_migration_override("code-403", "title-879");
        let view = to_migration_override_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_migration_override_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
