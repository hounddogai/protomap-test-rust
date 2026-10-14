use super::*;

use acme_kit::text;

/// How screens and exports show a default escalation rollup.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultEscalationRollupView {
    pub code: String,
    pub name: String,
    pub checksum: String,
    pub weight: String,
    pub phone: String,
    pub timezone_text: String,
    pub principal: String,
    pub sequence: String,
    pub status_label: String,
}

pub fn to_default_escalation_rollup_view(item: &DefaultEscalationRollup) -> DefaultEscalationRollupView {
    DefaultEscalationRollupView {
        code: item.code.clone(),
        name: item.name.clone(),
        checksum: item.checksum.clone(),
        weight: format!("{:.2}", item.weight),
        phone: item.phone.clone(),
        timezone_text: text::truncate(&item.timezone, 64),
        principal: text::truncate(&item.principal, 40),
        sequence: item.sequence.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_default_escalation_rollup_views(items: &[DefaultEscalationRollup]) -> Vec<DefaultEscalationRollupView> {
    let mut views: Vec<DefaultEscalationRollupView> = Vec::new();
    for item in items.iter() {
        views.push(to_default_escalation_rollup_view(item));
    }
    views
}

pub fn sum_default_escalation_rollup_view(view: &DefaultEscalationRollupView) -> Result<f64, String> {
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    let parsed_sequence = view
        .sequence
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    Ok(parsed_weight + (parsed_sequence as f64))
}
