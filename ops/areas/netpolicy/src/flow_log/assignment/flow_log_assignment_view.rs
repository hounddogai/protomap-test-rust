use super::*;

use acme_kit::text;

/// How screens and exports show a flow log assignment.
#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogAssignmentView {
    pub code_text: String,
    pub label_text: String,
    pub ceiling_text: String,
    pub rate: String,
    pub name: String,
    pub status_label: String,
}

pub fn to_flow_log_assignment_view(item: &FlowLogAssignment) -> FlowLogAssignmentView {
    FlowLogAssignmentView {
        code_text: item.code.clone(),
        label_text: item.label.clone(),
        ceiling_text: format!("{:.2}", item.ceiling),
        rate: format!("{:.2}", item.rate),
        name: text::truncate(&item.name, 80),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_flow_log_assignment_views(items: &[FlowLogAssignment]) -> Vec<FlowLogAssignmentView> {
    let mut views: Vec<FlowLogAssignmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_flow_log_assignment_view(item));
    }
    views
}

pub fn sum_flow_log_assignment_view(view: &FlowLogAssignmentView) -> Result<f64, String> {
    let parsed_ceiling = view
        .ceiling_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    Ok(parsed_ceiling + parsed_rate)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_flow_log_assignment("code-145", "label-267");
        let view = to_flow_log_assignment_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_flow_log_assignment_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
