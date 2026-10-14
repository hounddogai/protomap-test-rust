use super::*;

use acme_kit::text;

/// How screens and exports show a migration adjustment.
#[derive(Debug, Clone, PartialEq)]
pub struct MigrationAdjustmentView {
    pub code: String,
    pub title: String,
    pub source: String,
    pub attempts_text: String,
    pub limit_text: String,
    pub host: String,
    pub summary: String,
    pub status_label: String,
}

pub fn to_migration_adjustment_view(item: &MigrationAdjustment) -> MigrationAdjustmentView {
    MigrationAdjustmentView {
        code: item.code.clone(),
        title: text::truncate(&item.title, 40),
        source: text::truncate(&item.source, 40),
        attempts_text: item.attempts.to_string(),
        limit_text: item.limit.to_string(),
        host: item.host.clone(),
        summary: item.summary.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_migration_adjustment_views(items: &[MigrationAdjustment]) -> Vec<MigrationAdjustmentView> {
    let mut views: Vec<MigrationAdjustmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_migration_adjustment_view(item));
    }
    views
}

pub fn sum_migration_adjustment_view(view: &MigrationAdjustmentView) -> Result<f64, String> {
    let parsed_attempts = view
        .attempts_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    let parsed_limit = view
        .limit_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    Ok(parsed_attempts as f64 + (parsed_limit as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_migration_adjustment("code-112", "title-953");
        let view = to_migration_adjustment_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_migration_adjustment_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
