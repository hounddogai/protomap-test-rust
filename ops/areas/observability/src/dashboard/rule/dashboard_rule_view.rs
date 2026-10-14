use super::*;

use acme_kit::text;

/// How screens and exports show a dashboard rule.
#[derive(Debug, Clone, PartialEq)]
pub struct DashboardRuleView {
    pub code: String,
    pub name: String,
    pub tenant_id: String,
    pub country: String,
    pub quota_units: String,
    pub starts_at: String,
    pub scope: String,
    pub expires_at: String,
    pub approved: String,
    pub category: String,
    pub status_label: String,
}

pub fn to_dashboard_rule_view(item: &DashboardRule) -> DashboardRuleView {
    DashboardRuleView {
        code: text::truncate(&item.code, 120),
        name: item.name.clone(),
        tenant_id: text::truncate(&item.tenant_id, 40),
        country: item.country.clone(),
        quota_units: item.quota_units.to_string(),
        starts_at: item.starts_at.to_string(),
        scope: text::truncate(&item.scope, 120),
        expires_at: item.expires_at.to_string(),
        approved: format!("{}", item.approved),
        category: item.category.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_dashboard_rule_views(items: &[DashboardRule]) -> Vec<DashboardRuleView> {
    let mut views: Vec<DashboardRuleView> = Vec::new();
    for item in items.iter() {
        views.push(to_dashboard_rule_view(item));
    }
    views
}

pub fn sum_dashboard_rule_view(view: &DashboardRuleView) -> Result<f64, String> {
    let parsed_quota_units = view
        .quota_units
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    let parsed_expires_at = view
        .expires_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    Ok(parsed_quota_units as f64 + (parsed_starts_at as f64) + (parsed_expires_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_dashboard_rule("code-625", "name-387");
        let view = to_dashboard_rule_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_dashboard_rule_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
