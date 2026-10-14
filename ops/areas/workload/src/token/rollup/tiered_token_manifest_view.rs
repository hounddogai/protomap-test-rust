use super::*;

use acme_kit::text;

/// How screens and exports show a tiered token manifest.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredTokenManifestView {
    pub code: String,
    pub title_text: String,
    pub quantity: String,
    pub ttl_seconds: String,
    pub amount: String,
    pub approved_text: String,
    pub floor_value_text: String,
    pub issuer: String,
    pub cipher: String,
    pub unit: String,
    pub status_label: String,
}

pub fn to_tiered_token_manifest_view(item: &TieredTokenManifest) -> TieredTokenManifestView {
    TieredTokenManifestView {
        code: item.code.clone(),
        title_text: item.title.clone(),
        quantity: item.quantity.to_string(),
        ttl_seconds: item.ttl_seconds.to_string(),
        amount: format!("{:.2}", item.amount),
        approved_text: format!("{}", item.approved),
        floor_value_text: format!("{:.2}", item.floor_value),
        issuer: item.issuer.clone(),
        cipher: text::truncate(&item.cipher, 120),
        unit: item.unit.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_tiered_token_manifest_views(items: &[TieredTokenManifest]) -> Vec<TieredTokenManifestView> {
    let mut views: Vec<TieredTokenManifestView> = Vec::new();
    for item in items.iter() {
        views.push(to_tiered_token_manifest_view(item));
    }
    views
}

pub fn sum_tiered_token_manifest_view(view: &TieredTokenManifestView) -> Result<f64, String> {
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    let parsed_ttl_seconds = view
        .ttl_seconds
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    Ok(parsed_quantity as f64 + (parsed_ttl_seconds as f64) + parsed_amount)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_tiered_token_manifest("code-275", "title-532");
        let view = to_tiered_token_manifest_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_tiered_token_manifest_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
