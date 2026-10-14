use super::*;

use acme_kit::text;

/// How screens and exports show a region pin rollup.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinRollupView {
    pub code: String,
    pub label: String,
    pub country: String,
    pub notes: String,
    pub name_text: String,
    pub quantity: String,
    pub status_label: String,
}

pub fn to_region_pin_rollup_view(item: &RegionPinRollup) -> RegionPinRollupView {
    RegionPinRollupView {
        code: text::truncate(&item.code, 40),
        label: item.label.clone(),
        country: text::truncate(&item.country, 64),
        notes: item.notes.clone(),
        name_text: item.name.clone(),
        quantity: item.quantity.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_region_pin_rollup_views(items: &[RegionPinRollup]) -> Vec<RegionPinRollupView> {
    let mut views: Vec<RegionPinRollupView> = Vec::new();
    for item in items.iter() {
        views.push(to_region_pin_rollup_view(item));
    }
    views
}

pub fn sum_region_pin_rollup_view(view: &RegionPinRollupView) -> Result<f64, String> {
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
        let item = new_region_pin_rollup("code-17", "label-304");
        let view = to_region_pin_rollup_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_region_pin_rollup_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
