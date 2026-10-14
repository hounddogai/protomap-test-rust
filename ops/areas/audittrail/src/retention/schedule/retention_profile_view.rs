use super::*;

use acme_kit::text;

/// How screens and exports show a retention profile.
#[derive(Debug, Clone, PartialEq)]
pub struct RetentionProfileView {
    pub code: String,
    pub title_text: String,
    pub limit_text: String,
    pub ends_at: String,
    pub name: String,
    pub scope_text: String,
    pub quota_units: String,
    pub attempts: String,
    pub segment: String,
    pub status_label: String,
}

pub fn to_retention_profile_view(item: &RetentionProfile) -> RetentionProfileView {
    RetentionProfileView {
        code: text::truncate(&item.code, 120),
        title_text: item.title.clone(),
        limit_text: item.limit.to_string(),
        ends_at: item.ends_at.to_string(),
        name: item.name.clone(),
        scope_text: item.scope.clone(),
        quota_units: item.quota_units.to_string(),
        attempts: item.attempts.to_string(),
        segment: item.segment.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_retention_profile_views(items: &[RetentionProfile]) -> Vec<RetentionProfileView> {
    let mut views: Vec<RetentionProfileView> = Vec::new();
    for item in items.iter() {
        views.push(to_retention_profile_view(item));
    }
    views
}

pub fn sum_retention_profile_view(view: &RetentionProfileView) -> Result<f64, String> {
    let parsed_limit = view
        .limit_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    let parsed_ends_at = view
        .ends_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    let parsed_quota_units = view
        .quota_units
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    Ok(parsed_limit as f64 + (parsed_ends_at as f64) + (parsed_quota_units as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_retention_profile("code-186", "title-107");
        let view = to_retention_profile_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_retention_profile_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
