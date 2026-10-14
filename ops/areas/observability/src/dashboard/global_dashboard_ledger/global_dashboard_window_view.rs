use super::*;

use acme_kit::text;

/// How screens and exports show a global dashboard window.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalDashboardWindowView {
    pub code: String,
    pub title: String,
    pub minimum: String,
    pub channel_text: String,
    pub capacity: String,
    pub attempts_text: String,
    pub revision_text: String,
    pub status_label: String,
}

pub fn to_global_dashboard_window_view(item: &GlobalDashboardWindow) -> GlobalDashboardWindowView {
    GlobalDashboardWindowView {
        code: item.code.clone(),
        title: text::truncate(&item.title, 120),
        minimum: format!("{:.2}", item.minimum),
        channel_text: text::truncate(&item.channel, 80),
        capacity: item.capacity.to_string(),
        attempts_text: item.attempts.to_string(),
        revision_text: item.revision.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_global_dashboard_window_views(items: &[GlobalDashboardWindow]) -> Vec<GlobalDashboardWindowView> {
    let mut views: Vec<GlobalDashboardWindowView> = Vec::new();
    for item in items.iter() {
        views.push(to_global_dashboard_window_view(item));
    }
    views
}

pub fn sum_global_dashboard_window_view(view: &GlobalDashboardWindowView) -> Result<f64, String> {
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    let parsed_capacity = view
        .capacity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    let parsed_attempts = view
        .attempts_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    Ok(parsed_minimum + (parsed_capacity as f64) + (parsed_attempts as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_global_dashboard_window("code-420", "title-304");
        let view = to_global_dashboard_window_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_global_dashboard_window_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
