use super::*;

/// How screens and exports show a dashboard assignment.
#[derive(Debug, Clone, PartialEq)]
pub struct DashboardAssignmentView {
    pub code: String,
    pub name: String,
    pub quantity: String,
    pub capacity: String,
    pub port_text: String,
    pub version: String,
    pub status_label: String,
}

pub fn to_dashboard_assignment_view(item: &DashboardAssignment) -> DashboardAssignmentView {
    DashboardAssignmentView {
        code: item.code.clone(),
        name: item.name.clone(),
        quantity: item.quantity.to_string(),
        capacity: item.capacity.to_string(),
        port_text: item.port.to_string(),
        version: item.version.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_dashboard_assignment_views(items: &[DashboardAssignment]) -> Vec<DashboardAssignmentView> {
    let mut views: Vec<DashboardAssignmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_dashboard_assignment_view(item));
    }
    views
}

pub fn sum_dashboard_assignment_view(view: &DashboardAssignmentView) -> Result<f64, String> {
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    let parsed_capacity = view
        .capacity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    let parsed_port = view
        .port_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    Ok(parsed_quantity as f64 + (parsed_capacity as f64) + (parsed_port as f64))
}
