use super::*;

use acme_kit::text;

/// How screens and exports show a peering claim.
#[derive(Debug, Clone, PartialEq)]
pub struct PeeringClaimView {
    pub code: String,
    pub label: String,
    pub description: String,
    pub budget: String,
    pub visible: String,
    pub cipher: String,
    pub rotation_days: String,
    pub duration_minutes_text: String,
    pub timezone_text: String,
    pub owner_id_text: String,
    pub priority: String,
    pub status_label: String,
}

pub fn to_peering_claim_view(item: &PeeringClaim) -> PeeringClaimView {
    PeeringClaimView {
        code: item.code.clone(),
        label: item.label.clone(),
        description: text::truncate(&item.description, 40),
        budget: format!("{:.2}", item.budget),
        visible: format!("{}", item.visible),
        cipher: item.cipher.clone(),
        rotation_days: item.rotation_days.to_string(),
        duration_minutes_text: item.duration_minutes.to_string(),
        timezone_text: text::truncate(&item.timezone, 80),
        owner_id_text: text::truncate(&item.owner_id, 64),
        priority: item.priority.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_peering_claim_views(items: &[PeeringClaim]) -> Vec<PeeringClaimView> {
    let mut views: Vec<PeeringClaimView> = Vec::new();
    for item in items.iter() {
        views.push(to_peering_claim_view(item));
    }
    views
}

pub fn sum_peering_claim_view(view: &PeeringClaimView) -> Result<f64, String> {
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    let parsed_rotation_days = view
        .rotation_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    let parsed_duration_minutes = view
        .duration_minutes_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    Ok(parsed_budget + (parsed_rotation_days as f64) + (parsed_duration_minutes as f64))
}
