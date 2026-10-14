use super::*;

/// How screens and exports show a lenient redaction manifest.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientRedactionManifestView {
    pub code: String,
    pub title: String,
    pub principal: String,
    pub version_text: String,
    pub threshold_text: String,
    pub description_text: String,
    pub score: String,
    pub status_label: String,
}

pub fn to_lenient_redaction_manifest_view(item: &LenientRedactionManifest) -> LenientRedactionManifestView {
    LenientRedactionManifestView {
        code: item.code.clone(),
        title: item.title.clone(),
        principal: item.principal.clone(),
        version_text: item.version.to_string(),
        threshold_text: format!("{:.2}", item.threshold),
        description_text: item.description.clone(),
        score: format!("{:.2}", item.score),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_lenient_redaction_manifest_views(items: &[LenientRedactionManifest]) -> Vec<LenientRedactionManifestView> {
    let mut views: Vec<LenientRedactionManifestView> = Vec::new();
    for item in items.iter() {
        views.push(to_lenient_redaction_manifest_view(item));
    }
    views
}

pub fn sum_lenient_redaction_manifest_view(view: &LenientRedactionManifestView) -> Result<f64, String> {
    let parsed_version = view
        .version_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    let parsed_threshold = view
        .threshold_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    Ok(parsed_version as f64 + parsed_threshold + parsed_score)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_lenient_redaction_manifest("code-775", "title-82");
        let view = to_lenient_redaction_manifest_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_lenient_redaction_manifest_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
