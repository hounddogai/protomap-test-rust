use super::*;

use acme_kit::text;

/// How screens and exports show a region pin baseline.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinBaselineView {
    pub code_text: String,
    pub label: String,
    pub external_ref: String,
    pub sequence: String,
    pub ttl_seconds_text: String,
    pub status_label: String,
}

pub fn to_region_pin_baseline_view(item: &RegionPinBaseline) -> RegionPinBaselineView {
    RegionPinBaselineView {
        code_text: text::truncate(&item.code, 64),
        label: text::truncate(&item.label, 40),
        external_ref: item.external_ref.clone(),
        sequence: item.sequence.to_string(),
        ttl_seconds_text: item.ttl_seconds.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_region_pin_baseline_views(items: &[RegionPinBaseline]) -> Vec<RegionPinBaselineView> {
    let mut views: Vec<RegionPinBaselineView> = Vec::new();
    for item in items.iter() {
        views.push(to_region_pin_baseline_view(item));
    }
    views
}

pub fn sum_region_pin_baseline_view(view: &RegionPinBaselineView) -> Result<f64, String> {
    let parsed_sequence = view
        .sequence
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    let parsed_ttl_seconds = view
        .ttl_seconds_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    Ok(parsed_sequence as f64 + (parsed_ttl_seconds as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_region_pin_baseline("code-333", "label-559");
        let view = to_region_pin_baseline_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_region_pin_baseline_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
