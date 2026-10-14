use super::*;

use acme_kit::text;

/// How screens and exports show a manual canary segment.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualCanarySegmentView {
    pub code: String,
    pub name: String,
    pub channel: String,
    pub floor_value: String,
    pub quantity: String,
    pub grace_days: String,
    pub version: String,
    pub host_text: String,
    pub fingerprint_text: String,
    pub approved: String,
    pub status_label: String,
}

pub fn to_manual_canary_segment_view(item: &ManualCanarySegment) -> ManualCanarySegmentView {
    ManualCanarySegmentView {
        code: item.code.clone(),
        name: item.name.clone(),
        channel: text::truncate(&item.channel, 40),
        floor_value: format!("{:.2}", item.floor_value),
        quantity: item.quantity.to_string(),
        grace_days: item.grace_days.to_string(),
        version: item.version.to_string(),
        host_text: item.host.clone(),
        fingerprint_text: item.fingerprint.clone(),
        approved: format!("{}", item.approved),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_manual_canary_segment_views(items: &[ManualCanarySegment]) -> Vec<ManualCanarySegmentView> {
    let mut views: Vec<ManualCanarySegmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_manual_canary_segment_view(item));
    }
    views
}

pub fn sum_manual_canary_segment_view(view: &ManualCanarySegmentView) -> Result<f64, String> {
    let parsed_floor_value = view
        .floor_value
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    Ok(parsed_floor_value + (parsed_quantity as f64) + (parsed_grace_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_manual_canary_segment("code-412", "name-801");
        let view = to_manual_canary_segment_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_manual_canary_segment_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
