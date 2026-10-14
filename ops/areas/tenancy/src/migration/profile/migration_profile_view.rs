use super::*;

use acme_kit::text;

/// How screens and exports show a migration profile.
#[derive(Debug, Clone, PartialEq)]
pub struct MigrationProfileView {
    pub code: String,
    pub label: String,
    pub ttl_seconds: String,
    pub external_ref_text: String,
    pub starts_at: String,
    pub cost: String,
    pub threshold: String,
    pub status_label: String,
}

pub fn to_migration_profile_view(item: &MigrationProfile) -> MigrationProfileView {
    MigrationProfileView {
        code: text::truncate(&item.code, 64),
        label: item.label.clone(),
        ttl_seconds: item.ttl_seconds.to_string(),
        external_ref_text: item.external_ref.clone(),
        starts_at: item.starts_at.to_string(),
        cost: format!("{:.2}", item.cost),
        threshold: format!("{:.2}", item.threshold),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_migration_profile_views(items: &[MigrationProfile]) -> Vec<MigrationProfileView> {
    let mut views: Vec<MigrationProfileView> = Vec::new();
    for item in items.iter() {
        views.push(to_migration_profile_view(item));
    }
    views
}

pub fn sum_migration_profile_view(view: &MigrationProfileView) -> Result<f64, String> {
    let parsed_ttl_seconds = view
        .ttl_seconds
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    let parsed_cost = view
        .cost
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    Ok(parsed_ttl_seconds as f64 + (parsed_starts_at as f64) + parsed_cost)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_migration_profile("code-250", "label-387");
        let view = to_migration_profile_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_migration_profile_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
