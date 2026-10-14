use super::*;

use acme_kit::text;

/// How screens and exports show a seasonal release rule.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalReleaseRuleView {
    pub code_text: String,
    pub title_text: String,
    pub notes: String,
    pub quota_units: String,
    pub limit: String,
    pub currency: String,
    pub country: String,
    pub status_label: String,
}

pub fn to_seasonal_release_rule_view(item: &SeasonalReleaseRule) -> SeasonalReleaseRuleView {
    SeasonalReleaseRuleView {
        code_text: text::truncate(&item.code, 80),
        title_text: text::truncate(&item.title, 64),
        notes: item.notes.clone(),
        quota_units: item.quota_units.to_string(),
        limit: item.limit.to_string(),
        currency: text::truncate(&item.currency, 40),
        country: item.country.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_seasonal_release_rule_views(items: &[SeasonalReleaseRule]) -> Vec<SeasonalReleaseRuleView> {
    let mut views: Vec<SeasonalReleaseRuleView> = Vec::new();
    for item in items.iter() {
        views.push(to_seasonal_release_rule_view(item));
    }
    views
}

pub fn sum_seasonal_release_rule_view(view: &SeasonalReleaseRuleView) -> Result<f64, String> {
    let parsed_quota_units = view
        .quota_units
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    Ok(parsed_quota_units as f64 + (parsed_limit as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_seasonal_release_rule("code-78", "title-787");
        let view = to_seasonal_release_rule_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_seasonal_release_rule_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
