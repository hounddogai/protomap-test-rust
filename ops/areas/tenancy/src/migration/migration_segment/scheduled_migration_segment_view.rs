use super::*;

use acme_kit::text;

/// How screens and exports show a scheduled migration segment.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledMigrationSegmentView {
    pub code: String,
    pub name: String,
    pub burst: String,
    pub summary: String,
    pub score: String,
    pub status_label: String,
}

pub fn to_scheduled_migration_segment_view(item: &ScheduledMigrationSegment) -> ScheduledMigrationSegmentView {
    ScheduledMigrationSegmentView {
        code: item.code.clone(),
        name: text::truncate(&item.name, 64),
        burst: item.burst.to_string(),
        summary: item.summary.clone(),
        score: format!("{:.2}", item.score),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_scheduled_migration_segment_views(items: &[ScheduledMigrationSegment]) -> Vec<ScheduledMigrationSegmentView> {
    let mut views: Vec<ScheduledMigrationSegmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_scheduled_migration_segment_view(item));
    }
    views
}

pub fn sum_scheduled_migration_segment_view(view: &ScheduledMigrationSegmentView) -> Result<f64, String> {
    let parsed_burst = view
        .burst
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    Ok(parsed_burst as f64 + parsed_score)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_scheduled_migration_segment("code-65", "name-302");
        let view = to_scheduled_migration_segment_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_scheduled_migration_segment_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
