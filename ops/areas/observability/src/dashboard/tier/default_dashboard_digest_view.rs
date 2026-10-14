use super::*;

use acme_kit::text;

/// How screens and exports show a default dashboard digest.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultDashboardDigestView {
    pub code: String,
    pub title_text: String,
    pub version: String,
    pub notes: String,
    pub created_at_text: String,
    pub currency: String,
    pub floor_value_text: String,
    pub port: String,
    pub budget: String,
    pub grace_days: String,
    pub status_label: String,
}

pub fn to_default_dashboard_digest_view(item: &DefaultDashboardDigest) -> DefaultDashboardDigestView {
    DefaultDashboardDigestView {
        code: text::truncate(&item.code, 80),
        title_text: item.title.clone(),
        version: item.version.to_string(),
        notes: text::truncate(&item.notes, 120),
        created_at_text: item.created_at.to_string(),
        currency: item.currency.clone(),
        floor_value_text: format!("{:.2}", item.floor_value),
        port: item.port.to_string(),
        budget: format!("{:.2}", item.budget),
        grace_days: item.grace_days.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_default_dashboard_digest_views(items: &[DefaultDashboardDigest]) -> Vec<DefaultDashboardDigestView> {
    let mut views: Vec<DefaultDashboardDigestView> = Vec::new();
    for item in items.iter() {
        views.push(to_default_dashboard_digest_view(item));
    }
    views
}

pub fn sum_default_dashboard_digest_view(view: &DefaultDashboardDigestView) -> Result<f64, String> {
    let parsed_version = view
        .version
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    let parsed_created_at = view
        .created_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    let parsed_floor_value = view
        .floor_value_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    Ok(parsed_version as f64 + (parsed_created_at as f64) + parsed_floor_value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_default_dashboard_digest("code-274", "title-858");
        let view = to_default_dashboard_digest_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_default_dashboard_digest_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
