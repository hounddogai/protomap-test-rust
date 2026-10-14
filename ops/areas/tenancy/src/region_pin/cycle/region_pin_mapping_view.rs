use super::*;

use acme_kit::text;

/// How screens and exports show a region pin mapping.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinMappingView {
    pub code: String,
    pub label: String,
    pub category: String,
    pub weight_text: String,
    pub locale_text: String,
    pub status_label: String,
}

pub fn to_region_pin_mapping_view(item: &RegionPinMapping) -> RegionPinMappingView {
    RegionPinMappingView {
        code: item.code.clone(),
        label: text::truncate(&item.label, 40),
        category: item.category.clone(),
        weight_text: format!("{:.2}", item.weight),
        locale_text: item.locale.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_region_pin_mapping_views(items: &[RegionPinMapping]) -> Vec<RegionPinMappingView> {
    let mut views: Vec<RegionPinMappingView> = Vec::new();
    for item in items.iter() {
        views.push(to_region_pin_mapping_view(item));
    }
    views
}

pub fn sum_region_pin_mapping_view(view: &RegionPinMappingView) -> Result<f64, String> {
    let parsed_weight = view
        .weight_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    Ok(parsed_weight)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_region_pin_mapping("code-473", "label-800");
        let view = to_region_pin_mapping_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_region_pin_mapping_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
