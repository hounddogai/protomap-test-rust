use super::*;

/// How screens and exports show a grant manifest.
#[derive(Debug, Clone, PartialEq)]
pub struct GrantManifestView {
    pub code_text: String,
    pub title_text: String,
    pub rate: String,
    pub quantity: String,
    pub revision: String,
    pub external_ref: String,
    pub scope_text: String,
    pub burst: String,
    pub status_label: String,
}

pub fn to_grant_manifest_view(item: &GrantManifest) -> GrantManifestView {
    GrantManifestView {
        code_text: item.code.clone(),
        title_text: item.title.clone(),
        rate: format!("{:.2}", item.rate),
        quantity: item.quantity.to_string(),
        revision: item.revision.to_string(),
        external_ref: item.external_ref.clone(),
        scope_text: item.scope.clone(),
        burst: item.burst.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_grant_manifest_views(items: &[GrantManifest]) -> Vec<GrantManifestView> {
    let mut views: Vec<GrantManifestView> = Vec::new();
    for item in items.iter() {
        views.push(to_grant_manifest_view(item));
    }
    views
}

pub fn sum_grant_manifest_view(view: &GrantManifestView) -> Result<f64, String> {
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    let parsed_revision = view
        .revision
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    Ok(parsed_rate + (parsed_quantity as f64) + (parsed_revision as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_grant_manifest("code-526", "title-707");
        let view = to_grant_manifest_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_grant_manifest_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
