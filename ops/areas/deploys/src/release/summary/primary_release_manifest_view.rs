use super::*;

use acme_kit::text;

/// How screens and exports show a primary release manifest.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryReleaseManifestView {
    pub code_text: String,
    pub title: String,
    pub priority: String,
    pub ceiling: String,
    pub burst: String,
    pub country: String,
    pub fingerprint_text: String,
    pub approved: String,
    pub maximum: String,
    pub status_label: String,
}

pub fn to_primary_release_manifest_view(item: &PrimaryReleaseManifest) -> PrimaryReleaseManifestView {
    PrimaryReleaseManifestView {
        code_text: item.code.clone(),
        title: item.title.clone(),
        priority: item.priority.to_string(),
        ceiling: format!("{:.2}", item.ceiling),
        burst: item.burst.to_string(),
        country: text::truncate(&item.country, 120),
        fingerprint_text: item.fingerprint.clone(),
        approved: format!("{}", item.approved),
        maximum: format!("{:.2}", item.maximum),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_primary_release_manifest_views(items: &[PrimaryReleaseManifest]) -> Vec<PrimaryReleaseManifestView> {
    let mut views: Vec<PrimaryReleaseManifestView> = Vec::new();
    for item in items.iter() {
        views.push(to_primary_release_manifest_view(item));
    }
    views
}

pub fn sum_primary_release_manifest_view(view: &PrimaryReleaseManifestView) -> Result<f64, String> {
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    let parsed_ceiling = view
        .ceiling
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    let parsed_burst = view
        .burst
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    Ok(parsed_priority as f64 + parsed_ceiling + (parsed_burst as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_primary_release_manifest("code-262", "title-988");
        let view = to_primary_release_manifest_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_primary_release_manifest_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
