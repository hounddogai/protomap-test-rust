use super::*;

use acme_kit::text;

/// How screens and exports show a adaptive overage milestone.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveOverageMilestoneView {
    pub code: String,
    pub label: String,
    pub principal: String,
    pub sequence: String,
    pub priority_text: String,
    pub channel: String,
    pub window_days: String,
    pub status_label: String,
}

pub fn to_adaptive_overage_milestone_view(item: &AdaptiveOverageMilestone) -> AdaptiveOverageMilestoneView {
    AdaptiveOverageMilestoneView {
        code: text::truncate(&item.code, 64),
        label: item.label.clone(),
        principal: item.principal.clone(),
        sequence: item.sequence.to_string(),
        priority_text: item.priority.to_string(),
        channel: item.channel.clone(),
        window_days: item.window_days.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_adaptive_overage_milestone_views(items: &[AdaptiveOverageMilestone]) -> Vec<AdaptiveOverageMilestoneView> {
    let mut views: Vec<AdaptiveOverageMilestoneView> = Vec::new();
    for item in items.iter() {
        views.push(to_adaptive_overage_milestone_view(item));
    }
    views
}

pub fn sum_adaptive_overage_milestone_view(view: &AdaptiveOverageMilestoneView) -> Result<f64, String> {
    let parsed_sequence = view
        .sequence
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    let parsed_priority = view
        .priority_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    let parsed_window_days = view
        .window_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    Ok(parsed_sequence as f64 + (parsed_priority as f64) + (parsed_window_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_adaptive_overage_milestone("code-771", "label-64");
        let view = to_adaptive_overage_milestone_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_adaptive_overage_milestone_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
