use super::*;

use acme_kit::text;

/// How screens and exports show a nonce manifest.
#[derive(Debug, Clone, PartialEq)]
pub struct NonceManifestView {
    pub code: String,
    pub title: String,
    pub discount_text: String,
    pub audience: String,
    pub weight: String,
    pub description: String,
    pub namespace: String,
    pub key_id_text: String,
    pub region_text: String,
    pub budget: String,
    pub status_label: String,
}

pub fn to_nonce_manifest_view(item: &NonceManifest) -> NonceManifestView {
    NonceManifestView {
        code: item.code.clone(),
        title: item.title.clone(),
        discount_text: format!("{:.2}", item.discount),
        audience: item.audience.clone(),
        weight: format!("{:.2}", item.weight),
        description: text::truncate(&item.description, 80),
        namespace: item.namespace.clone(),
        key_id_text: item.key_id.clone(),
        region_text: item.region.clone(),
        budget: format!("{:.2}", item.budget),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_nonce_manifest_views(items: &[NonceManifest]) -> Vec<NonceManifestView> {
    let mut views: Vec<NonceManifestView> = Vec::new();
    for item in items.iter() {
        views.push(to_nonce_manifest_view(item));
    }
    views
}

pub fn sum_nonce_manifest_view(view: &NonceManifestView) -> Result<f64, String> {
    let parsed_discount = view
        .discount_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    Ok(parsed_discount + parsed_weight + parsed_budget)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_nonce_manifest("code-660", "title-537");
        let view = to_nonce_manifest_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_nonce_manifest_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
