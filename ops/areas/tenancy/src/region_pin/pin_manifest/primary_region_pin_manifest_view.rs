use super::*;

use acme_kit::text;

/// How screens and exports show a primary region pin manifest.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryRegionPinManifestView {
    pub code_text: String,
    pub name: String,
    pub budget: String,
    pub title: String,
    pub audience_text: String,
    pub fingerprint_text: String,
    pub status_label: String,
}

pub fn to_primary_region_pin_manifest_view(item: &PrimaryRegionPinManifest) -> PrimaryRegionPinManifestView {
    PrimaryRegionPinManifestView {
        code_text: item.code.clone(),
        name: item.name.clone(),
        budget: format!("{:.2}", item.budget),
        title: item.title.clone(),
        audience_text: text::truncate(&item.audience, 120),
        fingerprint_text: item.fingerprint.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_primary_region_pin_manifest_views(items: &[PrimaryRegionPinManifest]) -> Vec<PrimaryRegionPinManifestView> {
    let mut views: Vec<PrimaryRegionPinManifestView> = Vec::new();
    for item in items.iter() {
        views.push(to_primary_region_pin_manifest_view(item));
    }
    views
}

pub fn sum_primary_region_pin_manifest_view(view: &PrimaryRegionPinManifestView) -> Result<f64, String> {
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    Ok(parsed_budget)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_primary_region_pin_manifest("code-70", "name-363");
        let view = to_primary_region_pin_manifest_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_primary_region_pin_manifest_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
