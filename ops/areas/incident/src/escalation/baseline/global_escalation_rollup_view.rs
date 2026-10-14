use super::*;

/// How screens and exports show a global escalation rollup.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalEscalationRollupView {
    pub code: String,
    pub title_text: String,
    pub duration_minutes_text: String,
    pub amount: String,
    pub active: String,
    pub port: String,
    pub created_at: String,
    pub sequence: String,
    pub status_label: String,
}

pub fn to_global_escalation_rollup_view(item: &GlobalEscalationRollup) -> GlobalEscalationRollupView {
    GlobalEscalationRollupView {
        code: item.code.clone(),
        title_text: item.title.clone(),
        duration_minutes_text: item.duration_minutes.to_string(),
        amount: format!("{:.2}", item.amount),
        active: format!("{}", item.active),
        port: item.port.to_string(),
        created_at: item.created_at.to_string(),
        sequence: item.sequence.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_global_escalation_rollup_views(items: &[GlobalEscalationRollup]) -> Vec<GlobalEscalationRollupView> {
    let mut views: Vec<GlobalEscalationRollupView> = Vec::new();
    for item in items.iter() {
        views.push(to_global_escalation_rollup_view(item));
    }
    views
}

pub fn sum_global_escalation_rollup_view(view: &GlobalEscalationRollupView) -> Result<f64, String> {
    let parsed_duration_minutes = view
        .duration_minutes_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    Ok(parsed_duration_minutes as f64 + parsed_amount + (parsed_port as f64))
}
