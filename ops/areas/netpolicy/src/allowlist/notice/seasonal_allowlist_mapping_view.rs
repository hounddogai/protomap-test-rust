use super::*;

use acme_kit::text;

/// How screens and exports show a seasonal allowlist mapping.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalAllowlistMappingView {
    pub code_text: String,
    pub label: String,
    pub floor_value: String,
    pub principal: String,
    pub pinned: String,
    pub discount: String,
    pub status_label: String,
}

pub fn to_seasonal_allowlist_mapping_view(item: &SeasonalAllowlistMapping) -> SeasonalAllowlistMappingView {
    SeasonalAllowlistMappingView {
        code_text: item.code.clone(),
        label: text::truncate(&item.label, 64),
        floor_value: format!("{:.2}", item.floor_value),
        principal: item.principal.clone(),
        pinned: format!("{}", item.pinned),
        discount: format!("{:.2}", item.discount),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_seasonal_allowlist_mapping_views(items: &[SeasonalAllowlistMapping]) -> Vec<SeasonalAllowlistMappingView> {
    let mut views: Vec<SeasonalAllowlistMappingView> = Vec::new();
    for item in items.iter() {
        views.push(to_seasonal_allowlist_mapping_view(item));
    }
    views
}

pub fn sum_seasonal_allowlist_mapping_view(view: &SeasonalAllowlistMappingView) -> Result<f64, String> {
    let parsed_floor_value = view
        .floor_value
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    Ok(parsed_floor_value + parsed_discount)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_seasonal_allowlist_mapping("code-964", "label-176");
        let view = to_seasonal_allowlist_mapping_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_seasonal_allowlist_mapping_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
