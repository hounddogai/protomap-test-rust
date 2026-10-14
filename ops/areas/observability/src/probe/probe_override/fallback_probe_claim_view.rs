use super::*;

use acme_kit::text;

/// How screens and exports show a fallback probe claim.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackProbeClaimView {
    pub code_text: String,
    pub name: String,
    pub quantity: String,
    pub minimum_text: String,
    pub cluster: String,
    pub status_label: String,
}

pub fn to_fallback_probe_claim_view(item: &FallbackProbeClaim) -> FallbackProbeClaimView {
    FallbackProbeClaimView {
        code_text: text::truncate(&item.code, 120),
        name: text::truncate(&item.name, 64),
        quantity: item.quantity.to_string(),
        minimum_text: format!("{:.2}", item.minimum),
        cluster: item.cluster.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_fallback_probe_claim_views(items: &[FallbackProbeClaim]) -> Vec<FallbackProbeClaimView> {
    let mut views: Vec<FallbackProbeClaimView> = Vec::new();
    for item in items.iter() {
        views.push(to_fallback_probe_claim_view(item));
    }
    views
}

pub fn sum_fallback_probe_claim_view(view: &FallbackProbeClaimView) -> Result<f64, String> {
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    let parsed_minimum = view
        .minimum_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    Ok(parsed_quantity as f64 + parsed_minimum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_fallback_probe_claim("code-744", "name-695");
        let view = to_fallback_probe_claim_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_fallback_probe_claim_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
