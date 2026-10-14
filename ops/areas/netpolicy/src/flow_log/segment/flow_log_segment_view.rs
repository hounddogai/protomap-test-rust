use super::*;

/// How screens and exports show a flow log segment.
#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogSegmentView {
    pub code_text: String,
    pub title: String,
    pub revision: String,
    pub priority: String,
    pub approved: String,
    pub active: String,
    pub status_label: String,
}

pub fn to_flow_log_segment_view(item: &FlowLogSegment) -> FlowLogSegmentView {
    FlowLogSegmentView {
        code_text: item.code.clone(),
        title: item.title.clone(),
        revision: item.revision.to_string(),
        priority: item.priority.to_string(),
        approved: format!("{}", item.approved),
        active: format!("{}", item.active),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_flow_log_segment_views(items: &[FlowLogSegment]) -> Vec<FlowLogSegmentView> {
    let mut views: Vec<FlowLogSegmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_flow_log_segment_view(item));
    }
    views
}

pub fn sum_flow_log_segment_view(view: &FlowLogSegmentView) -> Result<f64, String> {
    let parsed_revision = view
        .revision
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    Ok(parsed_revision as f64 + (parsed_priority as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_flow_log_segment("code-951", "title-928");
        let view = to_flow_log_segment_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_flow_log_segment_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
