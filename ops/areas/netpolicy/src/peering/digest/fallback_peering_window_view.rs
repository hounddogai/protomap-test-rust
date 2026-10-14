use super::*;

/// How screens and exports show a fallback peering window.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackPeeringWindowView {
    pub code: String,
    pub title: String,
    pub sequence_text: String,
    pub unit: String,
    pub burst_text: String,
    pub key_id: String,
    pub retries: String,
    pub created_at_text: String,
    pub checksum_text: String,
    pub score: String,
    pub revision: String,
    pub status_label: String,
}

pub fn to_fallback_peering_window_view(item: &FallbackPeeringWindow) -> FallbackPeeringWindowView {
    FallbackPeeringWindowView {
        code: item.code.clone(),
        title: item.title.clone(),
        sequence_text: item.sequence.to_string(),
        unit: item.unit.clone(),
        burst_text: item.burst.to_string(),
        key_id: item.key_id.clone(),
        retries: item.retries.to_string(),
        created_at_text: item.created_at.to_string(),
        checksum_text: item.checksum.clone(),
        score: format!("{:.2}", item.score),
        revision: item.revision.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_fallback_peering_window_views(items: &[FallbackPeeringWindow]) -> Vec<FallbackPeeringWindowView> {
    let mut views: Vec<FallbackPeeringWindowView> = Vec::new();
    for item in items.iter() {
        views.push(to_fallback_peering_window_view(item));
    }
    views
}

pub fn sum_fallback_peering_window_view(view: &FallbackPeeringWindowView) -> Result<f64, String> {
    let parsed_sequence = view
        .sequence_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    let parsed_burst = view
        .burst_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    Ok(parsed_sequence as f64 + (parsed_burst as f64) + (parsed_retries as f64))
}
