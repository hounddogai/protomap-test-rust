use super::*;

use acme_kit::text;

/// How screens and exports show a strict health check milestone.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictHealthCheckMilestoneView {
    pub code: String,
    pub title: String,
    pub timezone: String,
    pub namespace: String,
    pub margin: String,
    pub status_label: String,
}

pub fn to_strict_health_check_milestone_view(item: &StrictHealthCheckMilestone) -> StrictHealthCheckMilestoneView {
    StrictHealthCheckMilestoneView {
        code: item.code.clone(),
        title: text::truncate(&item.title, 80),
        timezone: text::truncate(&item.timezone, 64),
        namespace: item.namespace.clone(),
        margin: format!("{:.2}", item.margin),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_strict_health_check_milestone_views(
    items: &[StrictHealthCheckMilestone],
) -> Vec<StrictHealthCheckMilestoneView> {
    let mut views: Vec<StrictHealthCheckMilestoneView> = Vec::new();
    for item in items.iter() {
        views.push(to_strict_health_check_milestone_view(item));
    }
    views
}

pub fn sum_strict_health_check_milestone_view(view: &StrictHealthCheckMilestoneView) -> Result<f64, String> {
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    Ok(parsed_margin)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_strict_health_check_milestone("code-326", "title-802");
        let view = to_strict_health_check_milestone_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_strict_health_check_milestone_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
