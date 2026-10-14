use super::*;

use acme_kit::text;

/// How screens and exports show a capped region pin manifest.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedRegionPinManifestView {
    pub code: String,
    pub name_text: String,
    pub sequence: String,
    pub region: String,
    pub attempts: String,
    pub principal: String,
    pub minimum: String,
    pub status_label: String,
}

pub fn to_capped_region_pin_manifest_view(item: &CappedRegionPinManifest) -> CappedRegionPinManifestView {
    CappedRegionPinManifestView {
        code: text::truncate(&item.code, 40),
        name_text: item.name.clone(),
        sequence: item.sequence.to_string(),
        region: item.region.clone(),
        attempts: item.attempts.to_string(),
        principal: text::truncate(&item.principal, 80),
        minimum: format!("{:.2}", item.minimum),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_capped_region_pin_manifest_views(items: &[CappedRegionPinManifest]) -> Vec<CappedRegionPinManifestView> {
    let mut views: Vec<CappedRegionPinManifestView> = Vec::new();
    for item in items.iter() {
        views.push(to_capped_region_pin_manifest_view(item));
    }
    views
}

pub fn sum_capped_region_pin_manifest_view(view: &CappedRegionPinManifestView) -> Result<f64, String> {
    let parsed_sequence = view
        .sequence
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    let parsed_attempts = view
        .attempts
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    Ok(parsed_sequence as f64 + (parsed_attempts as f64) + parsed_minimum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_capped_region_pin_manifest("code-805", "name-503");
        let view = to_capped_region_pin_manifest_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_capped_region_pin_manifest_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
