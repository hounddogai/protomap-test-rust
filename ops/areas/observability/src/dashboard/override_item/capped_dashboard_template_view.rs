use super::*;

use acme_kit::text;

/// How screens and exports show a capped dashboard template.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedDashboardTemplateView {
    pub code: String,
    pub title: String,
    pub ends_at: String,
    pub updated_at: String,
    pub limit: String,
    pub ttl_seconds: String,
    pub window_days: String,
    pub status_label: String,
}

pub fn to_capped_dashboard_template_view(item: &CappedDashboardTemplate) -> CappedDashboardTemplateView {
    CappedDashboardTemplateView {
        code: text::truncate(&item.code, 120),
        title: text::truncate(&item.title, 80),
        ends_at: item.ends_at.to_string(),
        updated_at: item.updated_at.to_string(),
        limit: item.limit.to_string(),
        ttl_seconds: item.ttl_seconds.to_string(),
        window_days: item.window_days.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_capped_dashboard_template_views(items: &[CappedDashboardTemplate]) -> Vec<CappedDashboardTemplateView> {
    let mut views: Vec<CappedDashboardTemplateView> = Vec::new();
    for item in items.iter() {
        views.push(to_capped_dashboard_template_view(item));
    }
    views
}

pub fn sum_capped_dashboard_template_view(view: &CappedDashboardTemplateView) -> Result<f64, String> {
    let parsed_ends_at = view
        .ends_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    let parsed_updated_at = view
        .updated_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    Ok(parsed_ends_at as f64 + (parsed_updated_at as f64) + (parsed_limit as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_capped_dashboard_template("code-535", "title-150");
        let view = to_capped_dashboard_template_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_capped_dashboard_template_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
