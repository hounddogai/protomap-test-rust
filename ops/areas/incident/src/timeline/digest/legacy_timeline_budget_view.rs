use super::*;

use acme_kit::text;

/// How screens and exports show a legacy timeline budget.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyTimelineBudgetView {
    pub code: String,
    pub title_text: String,
    pub source_text: String,
    pub host: String,
    pub quota_units: String,
    pub revision: String,
    pub channel_text: String,
    pub approved: String,
    pub issuer: String,
    pub category: String,
    pub duration_minutes: String,
    pub status_label: String,
}

pub fn to_legacy_timeline_budget_view(item: &LegacyTimelineBudget) -> LegacyTimelineBudgetView {
    LegacyTimelineBudgetView {
        code: item.code.clone(),
        title_text: text::truncate(&item.title, 64),
        source_text: item.source.clone(),
        host: text::truncate(&item.host, 64),
        quota_units: item.quota_units.to_string(),
        revision: item.revision.to_string(),
        channel_text: text::truncate(&item.channel, 120),
        approved: format!("{}", item.approved),
        issuer: item.issuer.clone(),
        category: item.category.clone(),
        duration_minutes: item.duration_minutes.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_legacy_timeline_budget_views(items: &[LegacyTimelineBudget]) -> Vec<LegacyTimelineBudgetView> {
    let mut views: Vec<LegacyTimelineBudgetView> = Vec::new();
    for item in items.iter() {
        views.push(to_legacy_timeline_budget_view(item));
    }
    views
}

pub fn sum_legacy_timeline_budget_view(view: &LegacyTimelineBudgetView) -> Result<f64, String> {
    let parsed_quota_units = view
        .quota_units
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    let parsed_revision = view
        .revision
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    let parsed_duration_minutes = view
        .duration_minutes
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    Ok(parsed_quota_units as f64 + (parsed_revision as f64) + (parsed_duration_minutes as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_legacy_timeline_budget("code-908", "title-284");
        let view = to_legacy_timeline_budget_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_legacy_timeline_budget_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
