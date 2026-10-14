use super::*;

use acme_kit::text;

/// How screens and exports show a seasonal allowlist milestone.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalAllowlistMilestoneView {
    pub code: String,
    pub label: String,
    pub limit: String,
    pub ttl_seconds: String,
    pub fingerprint: String,
    pub status_label: String,
}

pub fn to_seasonal_allowlist_milestone_view(item: &SeasonalAllowlistMilestone) -> SeasonalAllowlistMilestoneView {
    SeasonalAllowlistMilestoneView {
        code: text::truncate(&item.code, 120),
        label: item.label.clone(),
        limit: item.limit.to_string(),
        ttl_seconds: item.ttl_seconds.to_string(),
        fingerprint: item.fingerprint.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_seasonal_allowlist_milestone_views(
    items: &[SeasonalAllowlistMilestone],
) -> Vec<SeasonalAllowlistMilestoneView> {
    let mut views: Vec<SeasonalAllowlistMilestoneView> = Vec::new();
    for item in items.iter() {
        views.push(to_seasonal_allowlist_milestone_view(item));
    }
    views
}

pub fn sum_seasonal_allowlist_milestone_view(view: &SeasonalAllowlistMilestoneView) -> Result<f64, String> {
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    let parsed_ttl_seconds = view
        .ttl_seconds
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    Ok(parsed_limit as f64 + (parsed_ttl_seconds as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_seasonal_allowlist_milestone("code-45", "label-622");
        let view = to_seasonal_allowlist_milestone_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_seasonal_allowlist_milestone_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
