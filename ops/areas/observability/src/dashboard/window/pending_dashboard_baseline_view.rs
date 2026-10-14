use super::*;

/// How screens and exports show a pending dashboard baseline.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingDashboardBaselineView {
    pub code: String,
    pub title: String,
    pub updated_at_text: String,
    pub threshold: String,
    pub priority_text: String,
    pub issuer: String,
    pub status_label: String,
}

pub fn to_pending_dashboard_baseline_view(item: &PendingDashboardBaseline) -> PendingDashboardBaselineView {
    PendingDashboardBaselineView {
        code: item.code.clone(),
        title: item.title.clone(),
        updated_at_text: item.updated_at.to_string(),
        threshold: format!("{:.2}", item.threshold),
        priority_text: item.priority.to_string(),
        issuer: item.issuer.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_pending_dashboard_baseline_views(items: &[PendingDashboardBaseline]) -> Vec<PendingDashboardBaselineView> {
    let mut views: Vec<PendingDashboardBaselineView> = Vec::new();
    for item in items.iter() {
        views.push(to_pending_dashboard_baseline_view(item));
    }
    views
}

pub fn sum_pending_dashboard_baseline_view(view: &PendingDashboardBaselineView) -> Result<f64, String> {
    let parsed_updated_at = view
        .updated_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    let parsed_threshold = view
        .threshold
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    let parsed_priority = view
        .priority_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    Ok(parsed_updated_at as f64 + parsed_threshold + (parsed_priority as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_pending_dashboard_baseline("code-343", "title-518");
        let view = to_pending_dashboard_baseline_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_pending_dashboard_baseline_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
