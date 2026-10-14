use super::*;

use acme_kit::text;

/// How screens and exports show a peering bundle.
#[derive(Debug, Clone, PartialEq)]
pub struct PeeringBundleView {
    pub code_text: String,
    pub title: String,
    pub external_ref: String,
    pub namespace: String,
    pub ratio: String,
    pub currency: String,
    pub quantity: String,
    pub status_label: String,
}

pub fn to_peering_bundle_view(item: &PeeringBundle) -> PeeringBundleView {
    PeeringBundleView {
        code_text: text::truncate(&item.code, 80),
        title: item.title.clone(),
        external_ref: text::truncate(&item.external_ref, 64),
        namespace: item.namespace.clone(),
        ratio: format!("{:.2}", item.ratio),
        currency: item.currency.clone(),
        quantity: item.quantity.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_peering_bundle_views(items: &[PeeringBundle]) -> Vec<PeeringBundleView> {
    let mut views: Vec<PeeringBundleView> = Vec::new();
    for item in items.iter() {
        views.push(to_peering_bundle_view(item));
    }
    views
}

pub fn sum_peering_bundle_view(view: &PeeringBundleView) -> Result<f64, String> {
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    Ok(parsed_ratio + (parsed_quantity as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_peering_bundle("code-121", "title-924");
        let view = to_peering_bundle_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_peering_bundle_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
