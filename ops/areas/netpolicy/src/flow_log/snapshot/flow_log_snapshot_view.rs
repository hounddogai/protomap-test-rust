use super::*;

use acme_kit::text;

/// How screens and exports show a flow log snapshot.
#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogSnapshotView {
    pub code: String,
    pub title_text: String,
    pub priority: String,
    pub segment: String,
    pub name: String,
    pub approved: String,
    pub fingerprint: String,
    pub status_label: String,
}

pub fn to_flow_log_snapshot_view(item: &FlowLogSnapshot) -> FlowLogSnapshotView {
    FlowLogSnapshotView {
        code: text::truncate(&item.code, 64),
        title_text: item.title.clone(),
        priority: item.priority.to_string(),
        segment: item.segment.clone(),
        name: item.name.clone(),
        approved: format!("{}", item.approved),
        fingerprint: item.fingerprint.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_flow_log_snapshot_views(items: &[FlowLogSnapshot]) -> Vec<FlowLogSnapshotView> {
    let mut views: Vec<FlowLogSnapshotView> = Vec::new();
    for item in items.iter() {
        views.push(to_flow_log_snapshot_view(item));
    }
    views
}

pub fn sum_flow_log_snapshot_view(view: &FlowLogSnapshotView) -> Result<f64, String> {
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    Ok(parsed_priority as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_flow_log_snapshot("code-687", "title-309");
        let view = to_flow_log_snapshot_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_flow_log_snapshot_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
