use super::*;

use acme_kit::text;

/// How screens and exports show a dashboard schedule.
#[derive(Debug, Clone, PartialEq)]
pub struct DashboardScheduleView {
    pub code: String,
    pub name: String,
    pub limit: String,
    pub country: String,
    pub batch_size: String,
    pub status_label: String,
}

pub fn to_dashboard_schedule_view(item: &DashboardSchedule) -> DashboardScheduleView {
    DashboardScheduleView {
        code: item.code.clone(),
        name: text::truncate(&item.name, 120),
        limit: item.limit.to_string(),
        country: text::truncate(&item.country, 80),
        batch_size: item.batch_size.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_dashboard_schedule_views(items: &[DashboardSchedule]) -> Vec<DashboardScheduleView> {
    let mut views: Vec<DashboardScheduleView> = Vec::new();
    for item in items.iter() {
        views.push(to_dashboard_schedule_view(item));
    }
    views
}

pub fn sum_dashboard_schedule_view(view: &DashboardScheduleView) -> Result<f64, String> {
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    let parsed_batch_size = view
        .batch_size
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    Ok(parsed_limit as f64 + (parsed_batch_size as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_dashboard_schedule("code-329", "name-505");
        let view = to_dashboard_schedule_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_dashboard_schedule_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
