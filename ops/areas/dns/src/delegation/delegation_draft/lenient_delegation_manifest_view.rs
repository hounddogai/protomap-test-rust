use super::*;

use acme_kit::text;

/// How screens and exports show a lenient delegation manifest.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientDelegationManifestView {
    pub code: String,
    pub title: String,
    pub limit_text: String,
    pub batch_size_text: String,
    pub approved_text: String,
    pub status_label: String,
}

pub fn to_lenient_delegation_manifest_view(item: &LenientDelegationManifest) -> LenientDelegationManifestView {
    LenientDelegationManifestView {
        code: item.code.clone(),
        title: text::truncate(&item.title, 120),
        limit_text: item.limit.to_string(),
        batch_size_text: item.batch_size.to_string(),
        approved_text: format!("{}", item.approved),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_lenient_delegation_manifest_views(items: &[LenientDelegationManifest]) -> Vec<LenientDelegationManifestView> {
    let mut views: Vec<LenientDelegationManifestView> = Vec::new();
    for item in items.iter() {
        views.push(to_lenient_delegation_manifest_view(item));
    }
    views
}

pub fn sum_lenient_delegation_manifest_view(view: &LenientDelegationManifestView) -> Result<f64, String> {
    let parsed_limit = view
        .limit_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    let parsed_batch_size = view
        .batch_size_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    Ok(parsed_limit as f64 + (parsed_batch_size as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_lenient_delegation_manifest("code-153", "title-466");
        let view = to_lenient_delegation_manifest_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_lenient_delegation_manifest_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
