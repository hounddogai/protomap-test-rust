use super::*;

use acme_kit::text;

/// How screens and exports show a legacy rollback baseline.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyRollbackBaselineView {
    pub code: String,
    pub title: String,
    pub updated_at: String,
    pub active: String,
    pub timezone_text: String,
    pub maximum_text: String,
    pub country_text: String,
    pub status_label: String,
}

pub fn to_legacy_rollback_baseline_view(item: &LegacyRollbackBaseline) -> LegacyRollbackBaselineView {
    LegacyRollbackBaselineView {
        code: text::truncate(&item.code, 120),
        title: item.title.clone(),
        updated_at: item.updated_at.to_string(),
        active: format!("{}", item.active),
        timezone_text: item.timezone.clone(),
        maximum_text: format!("{:.2}", item.maximum),
        country_text: item.country.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_legacy_rollback_baseline_views(items: &[LegacyRollbackBaseline]) -> Vec<LegacyRollbackBaselineView> {
    let mut views: Vec<LegacyRollbackBaselineView> = Vec::new();
    for item in items.iter() {
        views.push(to_legacy_rollback_baseline_view(item));
    }
    views
}

pub fn sum_legacy_rollback_baseline_view(view: &LegacyRollbackBaselineView) -> Result<f64, String> {
    let parsed_updated_at = view
        .updated_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    let parsed_maximum = view
        .maximum_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    Ok(parsed_updated_at as f64 + parsed_maximum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_legacy_rollback_baseline("code-732", "title-18");
        let view = to_legacy_rollback_baseline_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_legacy_rollback_baseline_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
