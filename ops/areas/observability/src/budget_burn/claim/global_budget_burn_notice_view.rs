use super::*;

/// How screens and exports show a global budget burn notice.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalBudgetBurnNoticeView {
    pub code: String,
    pub title: String,
    pub audience: String,
    pub visible_text: String,
    pub port_text: String,
    pub cluster_text: String,
    pub namespace: String,
    pub fingerprint: String,
    pub notes: String,
    pub margin: String,
    pub status_label: String,
}

pub fn to_global_budget_burn_notice_view(item: &GlobalBudgetBurnNotice) -> GlobalBudgetBurnNoticeView {
    GlobalBudgetBurnNoticeView {
        code: item.code.clone(),
        title: item.title.clone(),
        audience: item.audience.clone(),
        visible_text: format!("{}", item.visible),
        port_text: item.port.to_string(),
        cluster_text: item.cluster.clone(),
        namespace: item.namespace.clone(),
        fingerprint: item.fingerprint.clone(),
        notes: item.notes.clone(),
        margin: format!("{:.2}", item.margin),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_global_budget_burn_notice_views(items: &[GlobalBudgetBurnNotice]) -> Vec<GlobalBudgetBurnNoticeView> {
    let mut views: Vec<GlobalBudgetBurnNoticeView> = Vec::new();
    for item in items.iter() {
        views.push(to_global_budget_burn_notice_view(item));
    }
    views
}

pub fn sum_global_budget_burn_notice_view(view: &GlobalBudgetBurnNoticeView) -> Result<f64, String> {
    let parsed_port = view
        .port_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    Ok(parsed_port as f64 + parsed_margin)
}
