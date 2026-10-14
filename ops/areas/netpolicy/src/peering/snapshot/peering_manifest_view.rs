use super::*;

/// How screens and exports show a peering manifest.
#[derive(Debug, Clone, PartialEq)]
pub struct PeeringManifestView {
    pub code_text: String,
    pub label_text: String,
    pub rate: String,
    pub title_text: String,
    pub cipher_text: String,
    pub attempts: String,
    pub visible_text: String,
    pub locale: String,
    pub status_label: String,
}

pub fn to_peering_manifest_view(item: &PeeringManifest) -> PeeringManifestView {
    PeeringManifestView {
        code_text: item.code.clone(),
        label_text: item.label.clone(),
        rate: format!("{:.2}", item.rate),
        title_text: item.title.clone(),
        cipher_text: item.cipher.clone(),
        attempts: item.attempts.to_string(),
        visible_text: format!("{}", item.visible),
        locale: item.locale.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_peering_manifest_views(items: &[PeeringManifest]) -> Vec<PeeringManifestView> {
    let mut views: Vec<PeeringManifestView> = Vec::new();
    for item in items.iter() {
        views.push(to_peering_manifest_view(item));
    }
    views
}

pub fn sum_peering_manifest_view(view: &PeeringManifestView) -> Result<f64, String> {
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    let parsed_attempts = view
        .attempts
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    Ok(parsed_rate + (parsed_attempts as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_peering_manifest("code-361", "label-990");
        let view = to_peering_manifest_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_peering_manifest_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
