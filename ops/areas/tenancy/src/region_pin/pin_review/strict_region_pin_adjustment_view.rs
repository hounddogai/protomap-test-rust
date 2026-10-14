use super::*;

use acme_kit::text;

/// How screens and exports show a strict region pin adjustment.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictRegionPinAdjustmentView {
    pub code_text: String,
    pub label: String,
    pub maximum: String,
    pub fingerprint_text: String,
    pub segment: String,
    pub approved: String,
    pub checksum_text: String,
    pub status_label: String,
}

pub fn to_strict_region_pin_adjustment_view(item: &StrictRegionPinAdjustment) -> StrictRegionPinAdjustmentView {
    StrictRegionPinAdjustmentView {
        code_text: text::truncate(&item.code, 80),
        label: text::truncate(&item.label, 64),
        maximum: format!("{:.2}", item.maximum),
        fingerprint_text: text::truncate(&item.fingerprint, 80),
        segment: item.segment.clone(),
        approved: format!("{}", item.approved),
        checksum_text: item.checksum.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_strict_region_pin_adjustment_views(
    items: &[StrictRegionPinAdjustment],
) -> Vec<StrictRegionPinAdjustmentView> {
    let mut views: Vec<StrictRegionPinAdjustmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_strict_region_pin_adjustment_view(item));
    }
    views
}

pub fn sum_strict_region_pin_adjustment_view(view: &StrictRegionPinAdjustmentView) -> Result<f64, String> {
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    Ok(parsed_maximum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_strict_region_pin_adjustment("code-713", "label-593");
        let view = to_strict_region_pin_adjustment_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_strict_region_pin_adjustment_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
