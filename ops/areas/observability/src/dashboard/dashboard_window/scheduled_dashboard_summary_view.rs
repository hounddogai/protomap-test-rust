use super::*;

use acme_kit::text;

/// How screens and exports show a scheduled dashboard summary.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledDashboardSummaryView {
    pub code: String,
    pub name_text: String,
    pub discount: String,
    pub revision: String,
    pub timezone_text: String,
    pub namespace_text: String,
    pub window_days: String,
    pub margin_text: String,
    pub visible: String,
    pub ratio: String,
    pub owner_id_text: String,
    pub status_label: String,
}

pub fn to_scheduled_dashboard_summary_view(item: &ScheduledDashboardSummary) -> ScheduledDashboardSummaryView {
    ScheduledDashboardSummaryView {
        code: text::truncate(&item.code, 40),
        name_text: item.name.clone(),
        discount: format!("{:.2}", item.discount),
        revision: item.revision.to_string(),
        timezone_text: item.timezone.clone(),
        namespace_text: item.namespace.clone(),
        window_days: item.window_days.to_string(),
        margin_text: format!("{:.2}", item.margin),
        visible: format!("{}", item.visible),
        ratio: format!("{:.2}", item.ratio),
        owner_id_text: item.owner_id.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_scheduled_dashboard_summary_views(items: &[ScheduledDashboardSummary]) -> Vec<ScheduledDashboardSummaryView> {
    let mut views: Vec<ScheduledDashboardSummaryView> = Vec::new();
    for item in items.iter() {
        views.push(to_scheduled_dashboard_summary_view(item));
    }
    views
}

pub fn sum_scheduled_dashboard_summary_view(view: &ScheduledDashboardSummaryView) -> Result<f64, String> {
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    let parsed_revision = view
        .revision
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    let parsed_window_days = view
        .window_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    Ok(parsed_discount + (parsed_revision as f64) + (parsed_window_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_scheduled_dashboard_summary("code-11", "name-272");
        let view = to_scheduled_dashboard_summary_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_scheduled_dashboard_summary_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
