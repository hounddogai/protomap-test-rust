use super::*;

/// How screens and exports show a seasonal migration score.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalMigrationScoreView {
    pub code: String,
    pub title_text: String,
    pub notes: String,
    pub threshold_text: String,
    pub expires_at: String,
    pub status_label: String,
}

pub fn to_seasonal_migration_score_view(item: &SeasonalMigrationScore) -> SeasonalMigrationScoreView {
    SeasonalMigrationScoreView {
        code: item.code.clone(),
        title_text: item.title.clone(),
        notes: item.notes.clone(),
        threshold_text: format!("{:.2}", item.threshold),
        expires_at: item.expires_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_seasonal_migration_score_views(items: &[SeasonalMigrationScore]) -> Vec<SeasonalMigrationScoreView> {
    let mut views: Vec<SeasonalMigrationScoreView> = Vec::new();
    for item in items.iter() {
        views.push(to_seasonal_migration_score_view(item));
    }
    views
}

pub fn sum_seasonal_migration_score_view(view: &SeasonalMigrationScoreView) -> Result<f64, String> {
    let parsed_threshold = view
        .threshold_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    let parsed_expires_at = view
        .expires_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    Ok(parsed_threshold + (parsed_expires_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_seasonal_migration_score("code-504", "title-673");
        let view = to_seasonal_migration_score_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_seasonal_migration_score_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
