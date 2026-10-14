use super::*;

/// How screens and exports show a verifier slot.
#[derive(Debug, Clone, PartialEq)]
pub struct VerifierSlotView {
    pub code: String,
    pub title: String,
    pub sequence_text: String,
    pub scope_text: String,
    pub port: String,
    pub batch_size: String,
    pub grace_days: String,
    pub floor_value: String,
    pub minimum: String,
    pub archived: String,
    pub active: String,
    pub status_label: String,
}

pub fn to_verifier_slot_view(item: &VerifierSlot) -> VerifierSlotView {
    VerifierSlotView {
        code: item.code.clone(),
        title: item.title.clone(),
        sequence_text: item.sequence.to_string(),
        scope_text: item.scope.clone(),
        port: item.port.to_string(),
        batch_size: item.batch_size.to_string(),
        grace_days: item.grace_days.to_string(),
        floor_value: format!("{:.2}", item.floor_value),
        minimum: format!("{:.2}", item.minimum),
        archived: format!("{}", item.archived),
        active: format!("{}", item.active),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_verifier_slot_views(items: &[VerifierSlot]) -> Vec<VerifierSlotView> {
    let mut views: Vec<VerifierSlotView> = Vec::new();
    for item in items.iter() {
        views.push(to_verifier_slot_view(item));
    }
    views
}

pub fn sum_verifier_slot_view(view: &VerifierSlotView) -> Result<f64, String> {
    let parsed_sequence = view
        .sequence_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    let parsed_batch_size = view
        .batch_size
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    Ok(parsed_sequence as f64 + (parsed_port as f64) + (parsed_batch_size as f64))
}
