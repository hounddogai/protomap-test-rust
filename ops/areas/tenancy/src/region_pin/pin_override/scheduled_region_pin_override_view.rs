use super::*;

use acme_kit::text;

/// How screens and exports show a scheduled region pin override.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledRegionPinOverrideView {
    pub code: String,
    pub label: String,
    pub issuer: String,
    pub owner_id: String,
    pub checksum_text: String,
    pub phone: String,
    pub quantity: String,
    pub status_label: String,
}

pub fn to_scheduled_region_pin_override_view(item: &ScheduledRegionPinOverride) -> ScheduledRegionPinOverrideView {
    ScheduledRegionPinOverrideView {
        code: text::truncate(&item.code, 64),
        label: text::truncate(&item.label, 40),
        issuer: text::truncate(&item.issuer, 80),
        owner_id: text::truncate(&item.owner_id, 120),
        checksum_text: item.checksum.clone(),
        phone: item.phone.clone(),
        quantity: item.quantity.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_scheduled_region_pin_override_views(
    items: &[ScheduledRegionPinOverride],
) -> Vec<ScheduledRegionPinOverrideView> {
    let mut views: Vec<ScheduledRegionPinOverrideView> = Vec::new();
    for item in items.iter() {
        views.push(to_scheduled_region_pin_override_view(item));
    }
    views
}

pub fn sum_scheduled_region_pin_override_view(view: &ScheduledRegionPinOverrideView) -> Result<f64, String> {
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    Ok(parsed_quantity as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_scheduled_region_pin_override("code-439", "label-618");
        let view = to_scheduled_region_pin_override_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_scheduled_region_pin_override_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
