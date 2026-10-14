use super::*;

use acme_kit::text;

/// How screens and exports show a dashboard template.
#[derive(Debug, Clone, PartialEq)]
pub struct DashboardTemplateView {
    pub code: String,
    pub name: String,
    pub expires_at: String,
    pub archived: String,
    pub scope: String,
    pub active: String,
    pub batch_size: String,
    pub quota_units: String,
    pub status_label: String,
}

pub fn to_dashboard_template_view(item: &DashboardTemplate) -> DashboardTemplateView {
    DashboardTemplateView {
        code: text::truncate(&item.code, 80),
        name: item.name.clone(),
        expires_at: item.expires_at.to_string(),
        archived: format!("{}", item.archived),
        scope: item.scope.clone(),
        active: format!("{}", item.active),
        batch_size: item.batch_size.to_string(),
        quota_units: item.quota_units.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_dashboard_template_views(items: &[DashboardTemplate]) -> Vec<DashboardTemplateView> {
    let mut views: Vec<DashboardTemplateView> = Vec::new();
    for item in items.iter() {
        views.push(to_dashboard_template_view(item));
    }
    views
}

pub fn sum_dashboard_template_view(view: &DashboardTemplateView) -> Result<f64, String> {
    let parsed_expires_at = view
        .expires_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    let parsed_batch_size = view
        .batch_size
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    let parsed_quota_units = view
        .quota_units
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    Ok(parsed_expires_at as f64 + (parsed_batch_size as f64) + (parsed_quota_units as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_dashboard_template("code-19", "name-453");
        let view = to_dashboard_template_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_dashboard_template_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
