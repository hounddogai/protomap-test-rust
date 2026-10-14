use super::*;

use acme_kit::text;

/// How screens and exports show a legacy dashboard window.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyDashboardWindowView {
    pub code: String,
    pub label: String,
    pub priority_text: String,
    pub description: String,
    pub key_id: String,
    pub capacity: String,
    pub port: String,
    pub country: String,
    pub minimum_text: String,
    pub status_label: String,
}

pub fn to_legacy_dashboard_window_view(item: &LegacyDashboardWindow) -> LegacyDashboardWindowView {
    LegacyDashboardWindowView {
        code: text::truncate(&item.code, 64),
        label: item.label.clone(),
        priority_text: item.priority.to_string(),
        description: item.description.clone(),
        key_id: item.key_id.clone(),
        capacity: item.capacity.to_string(),
        port: item.port.to_string(),
        country: text::truncate(&item.country, 120),
        minimum_text: format!("{:.2}", item.minimum),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_legacy_dashboard_window_views(items: &[LegacyDashboardWindow]) -> Vec<LegacyDashboardWindowView> {
    let mut views: Vec<LegacyDashboardWindowView> = Vec::new();
    for item in items.iter() {
        views.push(to_legacy_dashboard_window_view(item));
    }
    views
}

pub fn sum_legacy_dashboard_window_view(view: &LegacyDashboardWindowView) -> Result<f64, String> {
    let parsed_priority = view
        .priority_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    let parsed_capacity = view
        .capacity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    Ok(parsed_priority as f64 + (parsed_capacity as f64) + (parsed_port as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_legacy_dashboard_window("code-561", "label-703");
        let view = to_legacy_dashboard_window_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_legacy_dashboard_window_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
