use super::*;

/// How screens and exports show a fallback federation summary.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackFederationSummaryView {
    pub code: String,
    pub label: String,
    pub expires_at: String,
    pub contact_email: String,
    pub threshold: String,
    pub budget: String,
    pub rate: String,
    pub updated_at_text: String,
    pub status_label: String,
}

pub fn to_fallback_federation_summary_view(item: &FallbackFederationSummary) -> FallbackFederationSummaryView {
    FallbackFederationSummaryView {
        code: item.code.clone(),
        label: item.label.clone(),
        expires_at: item.expires_at.to_string(),
        contact_email: item.contact_email.clone(),
        threshold: format!("{:.2}", item.threshold),
        budget: format!("{:.2}", item.budget),
        rate: format!("{:.2}", item.rate),
        updated_at_text: item.updated_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_fallback_federation_summary_views(items: &[FallbackFederationSummary]) -> Vec<FallbackFederationSummaryView> {
    let mut views: Vec<FallbackFederationSummaryView> = Vec::new();
    for item in items.iter() {
        views.push(to_fallback_federation_summary_view(item));
    }
    views
}

pub fn sum_fallback_federation_summary_view(view: &FallbackFederationSummaryView) -> Result<f64, String> {
    let parsed_expires_at = view
        .expires_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    let parsed_threshold = view
        .threshold
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    Ok(parsed_expires_at as f64 + parsed_threshold + parsed_budget)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_fallback_federation_summary("code-123", "label-938");
        let view = to_fallback_federation_summary_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_fallback_federation_summary_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
