use super::*;

/// How screens and exports show a fallback trust domain estimate.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackTrustDomainEstimateView {
    pub code: String,
    pub label: String,
    pub currency: String,
    pub limit: String,
    pub rotation_days: String,
    pub sequence_text: String,
    pub quota_units: String,
    pub retries: String,
    pub issuer_text: String,
    pub status_label: String,
}

pub fn to_fallback_trust_domain_estimate_view(item: &FallbackTrustDomainEstimate) -> FallbackTrustDomainEstimateView {
    FallbackTrustDomainEstimateView {
        code: item.code.clone(),
        label: item.label.clone(),
        currency: item.currency.clone(),
        limit: item.limit.to_string(),
        rotation_days: item.rotation_days.to_string(),
        sequence_text: item.sequence.to_string(),
        quota_units: item.quota_units.to_string(),
        retries: item.retries.to_string(),
        issuer_text: item.issuer.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_fallback_trust_domain_estimate_views(
    items: &[FallbackTrustDomainEstimate],
) -> Vec<FallbackTrustDomainEstimateView> {
    let mut views: Vec<FallbackTrustDomainEstimateView> = Vec::new();
    for item in items.iter() {
        views.push(to_fallback_trust_domain_estimate_view(item));
    }
    views
}

pub fn sum_fallback_trust_domain_estimate_view(view: &FallbackTrustDomainEstimateView) -> Result<f64, String> {
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    let parsed_rotation_days = view
        .rotation_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    let parsed_sequence = view
        .sequence_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    Ok(parsed_limit as f64 + (parsed_rotation_days as f64) + (parsed_sequence as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_fallback_trust_domain_estimate("code-50", "label-736");
        let view = to_fallback_trust_domain_estimate_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_fallback_trust_domain_estimate_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
