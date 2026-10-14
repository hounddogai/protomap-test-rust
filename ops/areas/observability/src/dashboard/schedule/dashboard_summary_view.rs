use super::*;

use acme_kit::text;

/// How screens and exports show a dashboard summary.
#[derive(Debug, Clone, PartialEq)]
pub struct DashboardSummaryView {
    pub code: String,
    pub label: String,
    pub unit: String,
    pub ratio: String,
    pub ends_at_text: String,
    pub window_days: String,
    pub attempts_text: String,
    pub contact_email: String,
    pub status_label: String,
}

pub fn to_dashboard_summary_view(item: &DashboardSummary) -> DashboardSummaryView {
    DashboardSummaryView {
        code: item.code.clone(),
        label: item.label.clone(),
        unit: item.unit.clone(),
        ratio: format!("{:.2}", item.ratio),
        ends_at_text: item.ends_at.to_string(),
        window_days: item.window_days.to_string(),
        attempts_text: item.attempts.to_string(),
        contact_email: text::truncate(&item.contact_email, 64),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_dashboard_summary_views(items: &[DashboardSummary]) -> Vec<DashboardSummaryView> {
    let mut views: Vec<DashboardSummaryView> = Vec::new();
    for item in items.iter() {
        views.push(to_dashboard_summary_view(item));
    }
    views
}

pub fn sum_dashboard_summary_view(view: &DashboardSummaryView) -> Result<f64, String> {
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    let parsed_ends_at = view
        .ends_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    let parsed_window_days = view
        .window_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    Ok(parsed_ratio + (parsed_ends_at as f64) + (parsed_window_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_dashboard_summary("code-609", "label-524");
        let view = to_dashboard_summary_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_dashboard_summary_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
