use super::*;

use acme_kit::text;

/// How screens and exports show a binding manifest.
#[derive(Debug, Clone, PartialEq)]
pub struct BindingManifestView {
    pub code: String,
    pub title: String,
    pub weight: String,
    pub rate: String,
    pub score_text: String,
    pub issuer_text: String,
    pub phone: String,
    pub status_label: String,
}

pub fn to_binding_manifest_view(item: &BindingManifest) -> BindingManifestView {
    BindingManifestView {
        code: text::truncate(&item.code, 64),
        title: item.title.clone(),
        weight: format!("{:.2}", item.weight),
        rate: format!("{:.2}", item.rate),
        score_text: format!("{:.2}", item.score),
        issuer_text: item.issuer.clone(),
        phone: item.phone.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_binding_manifest_views(items: &[BindingManifest]) -> Vec<BindingManifestView> {
    let mut views: Vec<BindingManifestView> = Vec::new();
    for item in items.iter() {
        views.push(to_binding_manifest_view(item));
    }
    views
}

pub fn sum_binding_manifest_view(view: &BindingManifestView) -> Result<f64, String> {
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    let parsed_score = view
        .score_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    Ok(parsed_weight + parsed_rate + parsed_score)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_binding_manifest("code-755", "title-938");
        let view = to_binding_manifest_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_binding_manifest_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
