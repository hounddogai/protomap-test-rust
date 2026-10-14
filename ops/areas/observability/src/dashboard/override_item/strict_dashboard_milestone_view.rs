use super::*;

use acme_kit::text;

/// How screens and exports show a strict dashboard milestone.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictDashboardMilestoneView {
    pub code_text: String,
    pub name: String,
    pub tenant_id: String,
    pub ratio: String,
    pub host: String,
    pub created_at: String,
    pub ends_at: String,
    pub status_label: String,
}

pub fn to_strict_dashboard_milestone_view(item: &StrictDashboardMilestone) -> StrictDashboardMilestoneView {
    StrictDashboardMilestoneView {
        code_text: text::truncate(&item.code, 40),
        name: item.name.clone(),
        tenant_id: item.tenant_id.clone(),
        ratio: format!("{:.2}", item.ratio),
        host: item.host.clone(),
        created_at: item.created_at.to_string(),
        ends_at: item.ends_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_strict_dashboard_milestone_views(items: &[StrictDashboardMilestone]) -> Vec<StrictDashboardMilestoneView> {
    let mut views: Vec<StrictDashboardMilestoneView> = Vec::new();
    for item in items.iter() {
        views.push(to_strict_dashboard_milestone_view(item));
    }
    views
}

pub fn sum_strict_dashboard_milestone_view(view: &StrictDashboardMilestoneView) -> Result<f64, String> {
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    let parsed_created_at = view
        .created_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    let parsed_ends_at = view
        .ends_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    Ok(parsed_ratio + (parsed_created_at as f64) + (parsed_ends_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_strict_dashboard_milestone("code-858", "name-607");
        let view = to_strict_dashboard_milestone_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_strict_dashboard_milestone_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
