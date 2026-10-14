use super::*;

/// How screens and exports show a fallback binding claim.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackBindingClaimView {
    pub code_text: String,
    pub label: String,
    pub fingerprint: String,
    pub threshold_text: String,
    pub active: String,
    pub status_label: String,
}

pub fn to_fallback_binding_claim_view(item: &FallbackBindingClaim) -> FallbackBindingClaimView {
    FallbackBindingClaimView {
        code_text: item.code.clone(),
        label: item.label.clone(),
        fingerprint: item.fingerprint.clone(),
        threshold_text: format!("{:.2}", item.threshold),
        active: format!("{}", item.active),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_fallback_binding_claim_views(items: &[FallbackBindingClaim]) -> Vec<FallbackBindingClaimView> {
    let mut views: Vec<FallbackBindingClaimView> = Vec::new();
    for item in items.iter() {
        views.push(to_fallback_binding_claim_view(item));
    }
    views
}

pub fn sum_fallback_binding_claim_view(view: &FallbackBindingClaimView) -> Result<f64, String> {
    let parsed_threshold = view
        .threshold_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    Ok(parsed_threshold)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_fallback_binding_claim("code-666", "label-538");
        let view = to_fallback_binding_claim_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_fallback_binding_claim_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
