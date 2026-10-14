use super::*;

use acme_kit::text;

/// How screens and exports show a fallback retention claim.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackRetentionClaimView {
    pub code: String,
    pub title: String,
    pub source: String,
    pub expires_at: String,
    pub starts_at_text: String,
    pub quantity: String,
    pub retries: String,
    pub unit: String,
    pub status_label: String,
}

pub fn to_fallback_retention_claim_view(item: &FallbackRetentionClaim) -> FallbackRetentionClaimView {
    FallbackRetentionClaimView {
        code: text::truncate(&item.code, 64),
        title: item.title.clone(),
        source: item.source.clone(),
        expires_at: item.expires_at.to_string(),
        starts_at_text: item.starts_at.to_string(),
        quantity: item.quantity.to_string(),
        retries: item.retries.to_string(),
        unit: item.unit.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_fallback_retention_claim_views(items: &[FallbackRetentionClaim]) -> Vec<FallbackRetentionClaimView> {
    let mut views: Vec<FallbackRetentionClaimView> = Vec::new();
    for item in items.iter() {
        views.push(to_fallback_retention_claim_view(item));
    }
    views
}

pub fn sum_fallback_retention_claim_view(view: &FallbackRetentionClaimView) -> Result<f64, String> {
    let parsed_expires_at = view
        .expires_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    let parsed_starts_at = view
        .starts_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    Ok(parsed_expires_at as f64 + (parsed_starts_at as f64) + (parsed_quantity as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_fallback_retention_claim("code-539", "title-264");
        let view = to_fallback_retention_claim_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_fallback_retention_claim_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
