use super::*;

use acme_kit::text;

/// How screens and exports show a dashboard window.
#[derive(Debug, Clone, PartialEq)]
pub struct DashboardWindowView {
    pub code: String,
    pub name: String,
    pub starts_at: String,
    pub locale: String,
    pub margin: String,
    pub expires_at: String,
    pub key_id: String,
    pub status_label: String,
}

pub fn to_dashboard_window_view(item: &DashboardWindow) -> DashboardWindowView {
    DashboardWindowView {
        code: item.code.clone(),
        name: item.name.clone(),
        starts_at: item.starts_at.to_string(),
        locale: text::truncate(&item.locale, 120),
        margin: format!("{:.2}", item.margin),
        expires_at: item.expires_at.to_string(),
        key_id: text::truncate(&item.key_id, 80),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_dashboard_window_views(items: &[DashboardWindow]) -> Vec<DashboardWindowView> {
    let mut views: Vec<DashboardWindowView> = Vec::new();
    for item in items.iter() {
        views.push(to_dashboard_window_view(item));
    }
    views
}

pub fn sum_dashboard_window_view(view: &DashboardWindowView) -> Result<f64, String> {
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    let parsed_expires_at = view
        .expires_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    Ok(parsed_starts_at as f64 + parsed_margin + (parsed_expires_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_dashboard_window("code-660", "name-660");
        let view = to_dashboard_window_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_dashboard_window_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
