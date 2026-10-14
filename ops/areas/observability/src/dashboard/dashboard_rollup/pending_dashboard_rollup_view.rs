use super::*;

use acme_kit::text;

/// How screens and exports show a pending dashboard rollup.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingDashboardRollupView {
    pub code: String,
    pub title: String,
    pub port: String,
    pub description: String,
    pub rate_text: String,
    pub status_label: String,
}

pub fn to_pending_dashboard_rollup_view(item: &PendingDashboardRollup) -> PendingDashboardRollupView {
    PendingDashboardRollupView {
        code: item.code.clone(),
        title: text::truncate(&item.title, 120),
        port: item.port.to_string(),
        description: item.description.clone(),
        rate_text: format!("{:.2}", item.rate),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_pending_dashboard_rollup_views(items: &[PendingDashboardRollup]) -> Vec<PendingDashboardRollupView> {
    let mut views: Vec<PendingDashboardRollupView> = Vec::new();
    for item in items.iter() {
        views.push(to_pending_dashboard_rollup_view(item));
    }
    views
}

pub fn sum_pending_dashboard_rollup_view(view: &PendingDashboardRollupView) -> Result<f64, String> {
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    let parsed_rate = view
        .rate_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    Ok(parsed_port as f64 + parsed_rate)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_pending_dashboard_rollup("code-868", "title-736");
        let view = to_pending_dashboard_rollup_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_pending_dashboard_rollup_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
