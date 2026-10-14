use super::*;

use acme_kit::text;

/// How screens and exports show a batched token manifest.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedTokenManifestView {
    pub code_text: String,
    pub name_text: String,
    pub quantity: String,
    pub rate: String,
    pub fingerprint_text: String,
    pub expires_at: String,
    pub locale: String,
    pub starts_at_text: String,
    pub cluster: String,
    pub limit: String,
    pub port_text: String,
    pub status_label: String,
}

pub fn to_batched_token_manifest_view(item: &BatchedTokenManifest) -> BatchedTokenManifestView {
    BatchedTokenManifestView {
        code_text: item.code.clone(),
        name_text: text::truncate(&item.name, 40),
        quantity: item.quantity.to_string(),
        rate: format!("{:.2}", item.rate),
        fingerprint_text: item.fingerprint.clone(),
        expires_at: item.expires_at.to_string(),
        locale: item.locale.clone(),
        starts_at_text: item.starts_at.to_string(),
        cluster: item.cluster.clone(),
        limit: item.limit.to_string(),
        port_text: item.port.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_batched_token_manifest_views(items: &[BatchedTokenManifest]) -> Vec<BatchedTokenManifestView> {
    let mut views: Vec<BatchedTokenManifestView> = Vec::new();
    for item in items.iter() {
        views.push(to_batched_token_manifest_view(item));
    }
    views
}

pub fn sum_batched_token_manifest_view(view: &BatchedTokenManifestView) -> Result<f64, String> {
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    let parsed_expires_at = view
        .expires_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    Ok(parsed_quantity as f64 + parsed_rate + (parsed_expires_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_batched_token_manifest("code-277", "name-223");
        let view = to_batched_token_manifest_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_batched_token_manifest_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
