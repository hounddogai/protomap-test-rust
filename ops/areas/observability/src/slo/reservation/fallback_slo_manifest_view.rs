use super::*;

use acme_kit::text;

/// How screens and exports show a fallback slo manifest.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackSloManifestView {
    pub code: String,
    pub name_text: String,
    pub title: String,
    pub namespace_text: String,
    pub discount_text: String,
    pub approved: String,
    pub created_at: String,
    pub status_label: String,
}

pub fn to_fallback_slo_manifest_view(item: &FallbackSloManifest) -> FallbackSloManifestView {
    FallbackSloManifestView {
        code: item.code.clone(),
        name_text: text::truncate(&item.name, 120),
        title: item.title.clone(),
        namespace_text: item.namespace.clone(),
        discount_text: format!("{:.2}", item.discount),
        approved: format!("{}", item.approved),
        created_at: item.created_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_fallback_slo_manifest_views(items: &[FallbackSloManifest]) -> Vec<FallbackSloManifestView> {
    let mut views: Vec<FallbackSloManifestView> = Vec::new();
    for item in items.iter() {
        views.push(to_fallback_slo_manifest_view(item));
    }
    views
}

pub fn sum_fallback_slo_manifest_view(view: &FallbackSloManifestView) -> Result<f64, String> {
    let parsed_discount = view
        .discount_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    let parsed_created_at = view
        .created_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    Ok(parsed_discount + (parsed_created_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_fallback_slo_manifest("code-215", "name-254");
        let view = to_fallback_slo_manifest_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_fallback_slo_manifest_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
