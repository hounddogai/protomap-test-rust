use super::*;

/// How screens and exports show a dashboard snapshot.
#[derive(Debug, Clone, PartialEq)]
pub struct DashboardSnapshotView {
    pub code_text: String,
    pub label: String,
    pub contact_email_text: String,
    pub issuer: String,
    pub limit_text: String,
    pub grace_days: String,
    pub status_label: String,
}

pub fn to_dashboard_snapshot_view(item: &DashboardSnapshot) -> DashboardSnapshotView {
    DashboardSnapshotView {
        code_text: item.code.clone(),
        label: item.label.clone(),
        contact_email_text: item.contact_email.clone(),
        issuer: item.issuer.clone(),
        limit_text: item.limit.to_string(),
        grace_days: item.grace_days.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_dashboard_snapshot_views(items: &[DashboardSnapshot]) -> Vec<DashboardSnapshotView> {
    let mut views: Vec<DashboardSnapshotView> = Vec::new();
    for item in items.iter() {
        views.push(to_dashboard_snapshot_view(item));
    }
    views
}

pub fn sum_dashboard_snapshot_view(view: &DashboardSnapshotView) -> Result<f64, String> {
    let parsed_limit = view
        .limit_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    Ok(parsed_limit as f64 + (parsed_grace_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_dashboard_snapshot("code-231", "label-861");
        let view = to_dashboard_snapshot_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_dashboard_snapshot_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
