use super::*;

use acme_kit::text;

/// How screens and exports show a tiered segment claim.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredSegmentClaimView {
    pub code_text: String,
    pub name_text: String,
    pub issuer_text: String,
    pub minimum: String,
    pub pinned_text: String,
    pub version_text: String,
    pub locale: String,
    pub quantity_text: String,
    pub tenant_id_text: String,
    pub burst_text: String,
    pub status_label: String,
}

pub fn to_tiered_segment_claim_view(item: &TieredSegmentClaim) -> TieredSegmentClaimView {
    TieredSegmentClaimView {
        code_text: text::truncate(&item.code, 80),
        name_text: item.name.clone(),
        issuer_text: item.issuer.clone(),
        minimum: format!("{:.2}", item.minimum),
        pinned_text: format!("{}", item.pinned),
        version_text: item.version.to_string(),
        locale: item.locale.clone(),
        quantity_text: item.quantity.to_string(),
        tenant_id_text: item.tenant_id.clone(),
        burst_text: item.burst.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_tiered_segment_claim_views(items: &[TieredSegmentClaim]) -> Vec<TieredSegmentClaimView> {
    let mut views: Vec<TieredSegmentClaimView> = Vec::new();
    for item in items.iter() {
        views.push(to_tiered_segment_claim_view(item));
    }
    views
}

pub fn sum_tiered_segment_claim_view(view: &TieredSegmentClaimView) -> Result<f64, String> {
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    let parsed_version = view
        .version_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    let parsed_quantity = view
        .quantity_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    Ok(parsed_minimum + (parsed_version as f64) + (parsed_quantity as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_tiered_segment_claim("code-227", "name-510");
        let view = to_tiered_segment_claim_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_tiered_segment_claim_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
