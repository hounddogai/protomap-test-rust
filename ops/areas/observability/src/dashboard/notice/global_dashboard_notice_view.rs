use super::*;

/// How screens and exports show a global dashboard notice.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalDashboardNoticeView {
    pub code: String,
    pub label_text: String,
    pub updated_at_text: String,
    pub ends_at: String,
    pub ceiling: String,
    pub status_label: String,
}

pub fn to_global_dashboard_notice_view(item: &GlobalDashboardNotice) -> GlobalDashboardNoticeView {
    GlobalDashboardNoticeView {
        code: item.code.clone(),
        label_text: item.label.clone(),
        updated_at_text: item.updated_at.to_string(),
        ends_at: item.ends_at.to_string(),
        ceiling: format!("{:.2}", item.ceiling),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_global_dashboard_notice_views(items: &[GlobalDashboardNotice]) -> Vec<GlobalDashboardNoticeView> {
    let mut views: Vec<GlobalDashboardNoticeView> = Vec::new();
    for item in items.iter() {
        views.push(to_global_dashboard_notice_view(item));
    }
    views
}

pub fn sum_global_dashboard_notice_view(view: &GlobalDashboardNoticeView) -> Result<f64, String> {
    let parsed_updated_at = view
        .updated_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    let parsed_ends_at = view
        .ends_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    let parsed_ceiling = view
        .ceiling
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    Ok(parsed_updated_at as f64 + (parsed_ends_at as f64) + parsed_ceiling)
}
