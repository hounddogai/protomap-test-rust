use super::*;

use acme_kit::text;

/// How screens and exports show a fallback region pin mapping.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackRegionPinMappingView {
    pub code: String,
    pub label: String,
    pub summary: String,
    pub created_at_text: String,
    pub source_text: String,
    pub title: String,
    pub ends_at: String,
    pub checksum: String,
    pub status_label: String,
}

pub fn to_fallback_region_pin_mapping_view(item: &FallbackRegionPinMapping) -> FallbackRegionPinMappingView {
    FallbackRegionPinMappingView {
        code: item.code.clone(),
        label: text::truncate(&item.label, 64),
        summary: item.summary.clone(),
        created_at_text: item.created_at.to_string(),
        source_text: item.source.clone(),
        title: item.title.clone(),
        ends_at: item.ends_at.to_string(),
        checksum: item.checksum.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_fallback_region_pin_mapping_views(items: &[FallbackRegionPinMapping]) -> Vec<FallbackRegionPinMappingView> {
    let mut views: Vec<FallbackRegionPinMappingView> = Vec::new();
    for item in items.iter() {
        views.push(to_fallback_region_pin_mapping_view(item));
    }
    views
}

pub fn sum_fallback_region_pin_mapping_view(view: &FallbackRegionPinMappingView) -> Result<f64, String> {
    let parsed_created_at = view
        .created_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    let parsed_ends_at = view
        .ends_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    Ok(parsed_created_at as f64 + (parsed_ends_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_fallback_region_pin_mapping("code-687", "label-671");
        let view = to_fallback_region_pin_mapping_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_fallback_region_pin_mapping_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
