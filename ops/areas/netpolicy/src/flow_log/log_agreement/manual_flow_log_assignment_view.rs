use super::*;

use acme_kit::text;

/// How screens and exports show a manual flow log assignment.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualFlowLogAssignmentView {
    pub code: String,
    pub title: String,
    pub host_text: String,
    pub burst_text: String,
    pub label_text: String,
    pub duration_minutes: String,
    pub ceiling: String,
    pub status_label: String,
}

pub fn to_manual_flow_log_assignment_view(item: &ManualFlowLogAssignment) -> ManualFlowLogAssignmentView {
    ManualFlowLogAssignmentView {
        code: text::truncate(&item.code, 120),
        title: item.title.clone(),
        host_text: text::truncate(&item.host, 64),
        burst_text: item.burst.to_string(),
        label_text: text::truncate(&item.label, 120),
        duration_minutes: item.duration_minutes.to_string(),
        ceiling: format!("{:.2}", item.ceiling),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_manual_flow_log_assignment_views(items: &[ManualFlowLogAssignment]) -> Vec<ManualFlowLogAssignmentView> {
    let mut views: Vec<ManualFlowLogAssignmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_manual_flow_log_assignment_view(item));
    }
    views
}

pub fn sum_manual_flow_log_assignment_view(view: &ManualFlowLogAssignmentView) -> Result<f64, String> {
    let parsed_burst = view
        .burst_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    let parsed_duration_minutes = view
        .duration_minutes
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    let parsed_ceiling = view
        .ceiling
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    Ok(parsed_burst as f64 + (parsed_duration_minutes as f64) + parsed_ceiling)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_manual_flow_log_assignment("code-4", "title-196");
        let view = to_manual_flow_log_assignment_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_manual_flow_log_assignment_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
