use super::*;

use acme_kit::text;

/// How screens and exports show a lenient query manifest.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientQueryManifestView {
    pub code_text: String,
    pub name: String,
    pub segment: String,
    pub priority: String,
    pub archived_text: String,
    pub principal_text: String,
    pub owner_id: String,
    pub status_label: String,
}

pub fn to_lenient_query_manifest_view(item: &LenientQueryManifest) -> LenientQueryManifestView {
    LenientQueryManifestView {
        code_text: text::truncate(&item.code, 80),
        name: text::truncate(&item.name, 120),
        segment: item.segment.clone(),
        priority: item.priority.to_string(),
        archived_text: format!("{}", item.archived),
        principal_text: item.principal.clone(),
        owner_id: item.owner_id.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_lenient_query_manifest_views(items: &[LenientQueryManifest]) -> Vec<LenientQueryManifestView> {
    let mut views: Vec<LenientQueryManifestView> = Vec::new();
    for item in items.iter() {
        views.push(to_lenient_query_manifest_view(item));
    }
    views
}

pub fn sum_lenient_query_manifest_view(view: &LenientQueryManifestView) -> Result<f64, String> {
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    Ok(parsed_priority as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_lenient_query_manifest("code-344", "name-466");
        let view = to_lenient_query_manifest_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_lenient_query_manifest_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
