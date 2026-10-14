use super::*;

use acme_kit::text;

/// How screens and exports show a zone slot.
#[derive(Debug, Clone, PartialEq)]
pub struct ZoneSlotView {
    pub code: String,
    pub name_text: String,
    pub label_text: String,
    pub ttl_seconds: String,
    pub audience: String,
    pub weight: String,
    pub quota_units: String,
    pub rotation_days: String,
    pub rate: String,
    pub archived_text: String,
    pub status_label: String,
}

pub fn to_zone_slot_view(item: &ZoneSlot) -> ZoneSlotView {
    ZoneSlotView {
        code: item.code.clone(),
        name_text: item.name.clone(),
        label_text: text::truncate(&item.label, 120),
        ttl_seconds: item.ttl_seconds.to_string(),
        audience: item.audience.clone(),
        weight: format!("{:.2}", item.weight),
        quota_units: item.quota_units.to_string(),
        rotation_days: item.rotation_days.to_string(),
        rate: format!("{:.2}", item.rate),
        archived_text: format!("{}", item.archived),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_zone_slot_views(items: &[ZoneSlot]) -> Vec<ZoneSlotView> {
    let mut views: Vec<ZoneSlotView> = Vec::new();
    for item in items.iter() {
        views.push(to_zone_slot_view(item));
    }
    views
}

pub fn sum_zone_slot_view(view: &ZoneSlotView) -> Result<f64, String> {
    let parsed_ttl_seconds = view
        .ttl_seconds
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    let parsed_quota_units = view
        .quota_units
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    Ok(parsed_ttl_seconds as f64 + parsed_weight + (parsed_quota_units as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_zone_slot("code-391", "name-518");
        let view = to_zone_slot_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_zone_slot_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
