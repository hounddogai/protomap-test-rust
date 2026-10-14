use super::*;

/// How screens and exports show a flow log mapping.
#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogMappingView {
    pub code_text: String,
    pub label: String,
    pub unit_text: String,
    pub starts_at: String,
    pub namespace: String,
    pub pinned: String,
    pub status_label: String,
}

pub fn to_flow_log_mapping_view(item: &FlowLogMapping) -> FlowLogMappingView {
    FlowLogMappingView {
        code_text: item.code.clone(),
        label: item.label.clone(),
        unit_text: item.unit.clone(),
        starts_at: item.starts_at.to_string(),
        namespace: item.namespace.clone(),
        pinned: format!("{}", item.pinned),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_flow_log_mapping_views(items: &[FlowLogMapping]) -> Vec<FlowLogMappingView> {
    let mut views: Vec<FlowLogMappingView> = Vec::new();
    for item in items.iter() {
        views.push(to_flow_log_mapping_view(item));
    }
    views
}

pub fn sum_flow_log_mapping_view(view: &FlowLogMappingView) -> Result<f64, String> {
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    Ok(parsed_starts_at as f64)
}
