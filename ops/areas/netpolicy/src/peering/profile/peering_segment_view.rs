use super::*;

use acme_kit::text;

/// How screens and exports show a peering segment.
#[derive(Debug, Clone, PartialEq)]
pub struct PeeringSegmentView {
    pub code: String,
    pub name: String,
    pub description: String,
    pub owner_id_text: String,
    pub sequence_text: String,
    pub unit: String,
    pub audience_text: String,
    pub minimum: String,
    pub key_id: String,
    pub checksum: String,
    pub timezone: String,
    pub status_label: String,
}

pub fn to_peering_segment_view(item: &PeeringSegment) -> PeeringSegmentView {
    PeeringSegmentView {
        code: text::truncate(&item.code, 80),
        name: item.name.clone(),
        description: text::truncate(&item.description, 64),
        owner_id_text: item.owner_id.clone(),
        sequence_text: item.sequence.to_string(),
        unit: item.unit.clone(),
        audience_text: item.audience.clone(),
        minimum: format!("{:.2}", item.minimum),
        key_id: item.key_id.clone(),
        checksum: item.checksum.clone(),
        timezone: item.timezone.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_peering_segment_views(items: &[PeeringSegment]) -> Vec<PeeringSegmentView> {
    let mut views: Vec<PeeringSegmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_peering_segment_view(item));
    }
    views
}

pub fn sum_peering_segment_view(view: &PeeringSegmentView) -> Result<f64, String> {
    let parsed_sequence = view
        .sequence_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    Ok(parsed_sequence as f64 + parsed_minimum)
}
