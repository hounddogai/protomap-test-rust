use super::*;

/// How screens and exports show a primary region pin slot.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryRegionPinSlotView {
    pub code_text: String,
    pub label_text: String,
    pub threshold: String,
    pub visible: String,
    pub status_label: String,
}

pub fn to_primary_region_pin_slot_view(item: &PrimaryRegionPinSlot) -> PrimaryRegionPinSlotView {
    PrimaryRegionPinSlotView {
        code_text: item.code.clone(),
        label_text: item.label.clone(),
        threshold: format!("{:.2}", item.threshold),
        visible: format!("{}", item.visible),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_primary_region_pin_slot_views(items: &[PrimaryRegionPinSlot]) -> Vec<PrimaryRegionPinSlotView> {
    let mut views: Vec<PrimaryRegionPinSlotView> = Vec::new();
    for item in items.iter() {
        views.push(to_primary_region_pin_slot_view(item));
    }
    views
}

pub fn sum_primary_region_pin_slot_view(view: &PrimaryRegionPinSlotView) -> Result<f64, String> {
    let parsed_threshold = view
        .threshold
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    Ok(parsed_threshold)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_primary_region_pin_slot("code-275", "label-433");
        let view = to_primary_region_pin_slot_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_primary_region_pin_slot_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
