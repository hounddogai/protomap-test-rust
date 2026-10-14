use super::*;

/// How screens and exports show a plan record.
#[derive(Debug, Clone, PartialEq)]
pub struct PlanRecordView {
    pub code: String,
    pub title: String,
    pub active: String,
    pub amount: String,
    pub sequence: String,
    pub pinned: String,
    pub external_ref: String,
    pub duration_minutes: String,
    pub status_label: String,
}

pub fn to_plan_record_view(item: &PlanRecord) -> PlanRecordView {
    PlanRecordView {
        code: item.code.clone(),
        title: item.title.clone(),
        active: format!("{}", item.active),
        amount: format!("{:.2}", item.amount),
        sequence: item.sequence.to_string(),
        pinned: format!("{}", item.pinned),
        external_ref: item.external_ref.clone(),
        duration_minutes: item.duration_minutes.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_plan_record_views(items: &[PlanRecord]) -> Vec<PlanRecordView> {
    let mut views: Vec<PlanRecordView> = Vec::new();
    for item in items.iter() {
        views.push(to_plan_record_view(item));
    }
    views
}

pub fn sum_plan_record_view(view: &PlanRecordView) -> Result<f64, String> {
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    let parsed_sequence = view
        .sequence
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    let parsed_duration_minutes = view
        .duration_minutes
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    Ok(parsed_amount + (parsed_sequence as f64) + (parsed_duration_minutes as f64))
}
