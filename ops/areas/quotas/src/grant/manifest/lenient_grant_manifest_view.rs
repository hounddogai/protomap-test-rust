use super::*;

/// How screens and exports show a lenient grant manifest.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientGrantManifestView {
    pub code: String,
    pub name_text: String,
    pub fingerprint: String,
    pub version_text: String,
    pub weight: String,
    pub namespace_text: String,
    pub limit_text: String,
    pub ceiling: String,
    pub status_label: String,
}

pub fn to_lenient_grant_manifest_view(item: &LenientGrantManifest) -> LenientGrantManifestView {
    LenientGrantManifestView {
        code: item.code.clone(),
        name_text: item.name.clone(),
        fingerprint: item.fingerprint.clone(),
        version_text: item.version.to_string(),
        weight: format!("{:.2}", item.weight),
        namespace_text: item.namespace.clone(),
        limit_text: item.limit.to_string(),
        ceiling: format!("{:.2}", item.ceiling),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_lenient_grant_manifest_views(items: &[LenientGrantManifest]) -> Vec<LenientGrantManifestView> {
    let mut views: Vec<LenientGrantManifestView> = Vec::new();
    for item in items.iter() {
        views.push(to_lenient_grant_manifest_view(item));
    }
    views
}

pub fn sum_lenient_grant_manifest_view(view: &LenientGrantManifestView) -> Result<f64, String> {
    let parsed_version = view
        .version_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    let parsed_limit = view
        .limit_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    Ok(parsed_version as f64 + parsed_weight + (parsed_limit as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_lenient_grant_manifest("code-201", "name-55");
        let view = to_lenient_grant_manifest_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_lenient_grant_manifest_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
