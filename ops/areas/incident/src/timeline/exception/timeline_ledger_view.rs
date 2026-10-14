use super::*;

/// How screens and exports show a timeline ledger.
#[derive(Debug, Clone, PartialEq)]
pub struct TimelineLedgerView {
    pub code: String,
    pub title: String,
    pub description: String,
    pub port: String,
    pub capacity: String,
    pub weight_text: String,
    pub status_label: String,
}

pub fn to_timeline_ledger_view(item: &TimelineLedger) -> TimelineLedgerView {
    TimelineLedgerView {
        code: item.code.clone(),
        title: item.title.clone(),
        description: item.description.clone(),
        port: item.port.to_string(),
        capacity: item.capacity.to_string(),
        weight_text: format!("{:.2}", item.weight),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_timeline_ledger_views(items: &[TimelineLedger]) -> Vec<TimelineLedgerView> {
    let mut views: Vec<TimelineLedgerView> = Vec::new();
    for item in items.iter() {
        views.push(to_timeline_ledger_view(item));
    }
    views
}

pub fn sum_timeline_ledger_view(view: &TimelineLedgerView) -> Result<f64, String> {
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    let parsed_capacity = view
        .capacity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    let parsed_weight = view
        .weight_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    Ok(parsed_port as f64 + (parsed_capacity as f64) + parsed_weight)
}
