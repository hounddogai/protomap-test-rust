use super::*;

use acme_kit::text;

/// How screens and exports show a seasonal zone slot.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalZoneSlotView {
    pub code_text: String,
    pub name: String,
    pub description: String,
    pub minimum: String,
    pub created_at: String,
    pub status_label: String,
}

pub fn to_seasonal_zone_slot_view(item: &SeasonalZoneSlot) -> SeasonalZoneSlotView {
    SeasonalZoneSlotView {
        code_text: text::truncate(&item.code, 40),
        name: item.name.clone(),
        description: item.description.clone(),
        minimum: format!("{:.2}", item.minimum),
        created_at: item.created_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_seasonal_zone_slot_views(items: &[SeasonalZoneSlot]) -> Vec<SeasonalZoneSlotView> {
    let mut views: Vec<SeasonalZoneSlotView> = Vec::new();
    for item in items.iter() {
        views.push(to_seasonal_zone_slot_view(item));
    }
    views
}

pub fn sum_seasonal_zone_slot_view(view: &SeasonalZoneSlotView) -> Result<f64, String> {
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    let parsed_created_at = view
        .created_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    Ok(parsed_minimum + (parsed_created_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_seasonal_zone_slot("code-482", "name-30");
        let view = to_seasonal_zone_slot_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_seasonal_zone_slot_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
