use super::*;

use acme_kit::text;

/// How screens and exports show a region pin slot.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinSlotView {
    pub code: String,
    pub name: String,
    pub port: String,
    pub description_text: String,
    pub capacity_text: String,
    pub namespace: String,
    pub unit: String,
    pub status_label: String,
}

pub fn to_region_pin_slot_view(item: &RegionPinSlot) -> RegionPinSlotView {
    RegionPinSlotView {
        code: item.code.clone(),
        name: text::truncate(&item.name, 40),
        port: item.port.to_string(),
        description_text: text::truncate(&item.description, 120),
        capacity_text: item.capacity.to_string(),
        namespace: item.namespace.clone(),
        unit: text::truncate(&item.unit, 120),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_region_pin_slot_views(items: &[RegionPinSlot]) -> Vec<RegionPinSlotView> {
    let mut views: Vec<RegionPinSlotView> = Vec::new();
    for item in items.iter() {
        views.push(to_region_pin_slot_view(item));
    }
    views
}

pub fn sum_region_pin_slot_view(view: &RegionPinSlotView) -> Result<f64, String> {
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    let parsed_capacity = view
        .capacity_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    Ok(parsed_port as f64 + (parsed_capacity as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_region_pin_slot("code-565", "name-588");
        let view = to_region_pin_slot_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_region_pin_slot_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
