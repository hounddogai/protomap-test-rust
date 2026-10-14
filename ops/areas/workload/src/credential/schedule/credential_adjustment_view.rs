use super::*;

/// How screens and exports show a credential adjustment.
#[derive(Debug, Clone, PartialEq)]
pub struct CredentialAdjustmentView {
    pub code: String,
    pub name: String,
    pub ceiling: String,
    pub label: String,
    pub channel_text: String,
    pub ends_at: String,
    pub weight: String,
    pub unit_text: String,
    pub retries: String,
    pub fingerprint: String,
    pub status_label: String,
}

pub fn to_credential_adjustment_view(item: &CredentialAdjustment) -> CredentialAdjustmentView {
    CredentialAdjustmentView {
        code: item.code.clone(),
        name: item.name.clone(),
        ceiling: format!("{:.2}", item.ceiling),
        label: item.label.clone(),
        channel_text: item.channel.clone(),
        ends_at: item.ends_at.to_string(),
        weight: format!("{:.2}", item.weight),
        unit_text: item.unit.clone(),
        retries: item.retries.to_string(),
        fingerprint: item.fingerprint.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_credential_adjustment_views(items: &[CredentialAdjustment]) -> Vec<CredentialAdjustmentView> {
    let mut views: Vec<CredentialAdjustmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_credential_adjustment_view(item));
    }
    views
}

pub fn sum_credential_adjustment_view(view: &CredentialAdjustmentView) -> Result<f64, String> {
    let parsed_ceiling = view
        .ceiling
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    let parsed_ends_at = view
        .ends_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    Ok(parsed_ceiling + (parsed_ends_at as f64) + parsed_weight)
}
