use super::*;

use acme_kit::text;

/// How screens and exports show a redaction milestone.
#[derive(Debug, Clone, PartialEq)]
pub struct RedactionMilestoneView {
    pub code: String,
    pub name: String,
    pub window_days: String,
    pub starts_at_text: String,
    pub summary: String,
    pub cost_text: String,
    pub maximum: String,
    pub cluster: String,
    pub rotation_days: String,
    pub status_label: String,
}

pub fn to_redaction_milestone_view(item: &RedactionMilestone) -> RedactionMilestoneView {
    RedactionMilestoneView {
        code: text::truncate(&item.code, 64),
        name: text::truncate(&item.name, 120),
        window_days: item.window_days.to_string(),
        starts_at_text: item.starts_at.to_string(),
        summary: text::truncate(&item.summary, 40),
        cost_text: format!("{:.2}", item.cost),
        maximum: format!("{:.2}", item.maximum),
        cluster: text::truncate(&item.cluster, 80),
        rotation_days: item.rotation_days.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_redaction_milestone_views(items: &[RedactionMilestone]) -> Vec<RedactionMilestoneView> {
    let mut views: Vec<RedactionMilestoneView> = Vec::new();
    for item in items.iter() {
        views.push(to_redaction_milestone_view(item));
    }
    views
}

pub fn sum_redaction_milestone_view(view: &RedactionMilestoneView) -> Result<f64, String> {
    let parsed_window_days = view
        .window_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    let parsed_starts_at = view
        .starts_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    let parsed_cost = view
        .cost_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    Ok(parsed_window_days as f64 + (parsed_starts_at as f64) + parsed_cost)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_redaction_milestone("code-329", "name-800");
        let view = to_redaction_milestone_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_redaction_milestone_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
