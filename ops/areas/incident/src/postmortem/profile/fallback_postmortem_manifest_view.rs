use super::*;

use acme_kit::text;

/// How screens and exports show a fallback postmortem manifest.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackPostmortemManifestView {
    pub code: String,
    pub name: String,
    pub limit: String,
    pub country: String,
    pub priority_text: String,
    pub label: String,
    pub namespace: String,
    pub status_label: String,
}

pub fn to_fallback_postmortem_manifest_view(item: &FallbackPostmortemManifest) -> FallbackPostmortemManifestView {
    FallbackPostmortemManifestView {
        code: text::truncate(&item.code, 64),
        name: item.name.clone(),
        limit: item.limit.to_string(),
        country: item.country.clone(),
        priority_text: item.priority.to_string(),
        label: item.label.clone(),
        namespace: item.namespace.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_fallback_postmortem_manifest_views(
    items: &[FallbackPostmortemManifest],
) -> Vec<FallbackPostmortemManifestView> {
    let mut views: Vec<FallbackPostmortemManifestView> = Vec::new();
    for item in items.iter() {
        views.push(to_fallback_postmortem_manifest_view(item));
    }
    views
}

pub fn sum_fallback_postmortem_manifest_view(view: &FallbackPostmortemManifestView) -> Result<f64, String> {
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    let parsed_priority = view
        .priority_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    Ok(parsed_limit as f64 + (parsed_priority as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_fallback_postmortem_manifest("code-477", "name-780");
        let view = to_fallback_postmortem_manifest_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_fallback_postmortem_manifest_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
