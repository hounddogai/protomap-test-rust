use super::*;

use acme_kit::text;

/// How screens and exports show a batched usage manifest.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedUsageManifestView {
    pub code_text: String,
    pub label: String,
    pub maximum: String,
    pub reviewer_text: String,
    pub quantity: String,
    pub description: String,
    pub status_label: String,
}

pub fn to_batched_usage_manifest_view(item: &BatchedUsageManifest) -> BatchedUsageManifestView {
    BatchedUsageManifestView {
        code_text: text::truncate(&item.code, 120),
        label: item.label.clone(),
        maximum: format!("{:.2}", item.maximum),
        reviewer_text: item.reviewer.clone(),
        quantity: item.quantity.to_string(),
        description: item.description.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_batched_usage_manifest_views(items: &[BatchedUsageManifest]) -> Vec<BatchedUsageManifestView> {
    let mut views: Vec<BatchedUsageManifestView> = Vec::new();
    for item in items.iter() {
        views.push(to_batched_usage_manifest_view(item));
    }
    views
}

pub fn sum_batched_usage_manifest_view(view: &BatchedUsageManifestView) -> Result<f64, String> {
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    Ok(parsed_maximum + (parsed_quantity as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_batched_usage_manifest("code-311", "label-386");
        let view = to_batched_usage_manifest_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_batched_usage_manifest_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
