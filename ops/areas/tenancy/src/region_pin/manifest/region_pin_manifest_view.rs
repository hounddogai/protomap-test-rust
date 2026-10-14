use super::*;

/// How screens and exports show a region pin manifest.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinManifestView {
    pub code: String,
    pub label_text: String,
    pub maximum: String,
    pub checksum_text: String,
    pub floor_value_text: String,
    pub status_label: String,
}

pub fn to_region_pin_manifest_view(item: &RegionPinManifest) -> RegionPinManifestView {
    RegionPinManifestView {
        code: item.code.clone(),
        label_text: item.label.clone(),
        maximum: format!("{:.2}", item.maximum),
        checksum_text: item.checksum.clone(),
        floor_value_text: format!("{:.2}", item.floor_value),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_region_pin_manifest_views(items: &[RegionPinManifest]) -> Vec<RegionPinManifestView> {
    let mut views: Vec<RegionPinManifestView> = Vec::new();
    for item in items.iter() {
        views.push(to_region_pin_manifest_view(item));
    }
    views
}

pub fn sum_region_pin_manifest_view(view: &RegionPinManifestView) -> Result<f64, String> {
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    let parsed_floor_value = view
        .floor_value_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    Ok(parsed_maximum + parsed_floor_value)
}
