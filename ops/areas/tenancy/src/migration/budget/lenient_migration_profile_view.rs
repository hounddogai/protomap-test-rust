use super::*;

use acme_kit::text;

/// How screens and exports show a lenient migration profile.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientMigrationProfileView {
    pub code: String,
    pub title_text: String,
    pub reviewer: String,
    pub capacity: String,
    pub ends_at_text: String,
    pub starts_at: String,
    pub threshold: String,
    pub ttl_seconds: String,
    pub limit: String,
    pub retries: String,
    pub cluster: String,
    pub status_label: String,
}

pub fn to_lenient_migration_profile_view(item: &LenientMigrationProfile) -> LenientMigrationProfileView {
    LenientMigrationProfileView {
        code: item.code.clone(),
        title_text: item.title.clone(),
        reviewer: text::truncate(&item.reviewer, 120),
        capacity: item.capacity.to_string(),
        ends_at_text: item.ends_at.to_string(),
        starts_at: item.starts_at.to_string(),
        threshold: format!("{:.2}", item.threshold),
        ttl_seconds: item.ttl_seconds.to_string(),
        limit: item.limit.to_string(),
        retries: item.retries.to_string(),
        cluster: item.cluster.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_lenient_migration_profile_views(items: &[LenientMigrationProfile]) -> Vec<LenientMigrationProfileView> {
    let mut views: Vec<LenientMigrationProfileView> = Vec::new();
    for item in items.iter() {
        views.push(to_lenient_migration_profile_view(item));
    }
    views
}

pub fn sum_lenient_migration_profile_view(view: &LenientMigrationProfileView) -> Result<f64, String> {
    let parsed_capacity = view
        .capacity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    let parsed_ends_at = view
        .ends_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    Ok(parsed_capacity as f64 + (parsed_ends_at as f64) + (parsed_starts_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_lenient_migration_profile("code-847", "title-420");
        let view = to_lenient_migration_profile_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_lenient_migration_profile_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
