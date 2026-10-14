use super::*;

/// How screens and exports show a fallback evidence agreement.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackEvidenceAgreementView {
    pub code: String,
    pub title: String,
    pub window_days_text: String,
    pub timezone: String,
    pub active: String,
    pub budget: String,
    pub maximum: String,
    pub quota_units: String,
    pub starts_at: String,
    pub status_label: String,
}

pub fn to_fallback_evidence_agreement_view(item: &FallbackEvidenceAgreement) -> FallbackEvidenceAgreementView {
    FallbackEvidenceAgreementView {
        code: item.code.clone(),
        title: item.title.clone(),
        window_days_text: item.window_days.to_string(),
        timezone: item.timezone.clone(),
        active: format!("{}", item.active),
        budget: format!("{:.2}", item.budget),
        maximum: format!("{:.2}", item.maximum),
        quota_units: item.quota_units.to_string(),
        starts_at: item.starts_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_fallback_evidence_agreement_views(items: &[FallbackEvidenceAgreement]) -> Vec<FallbackEvidenceAgreementView> {
    let mut views: Vec<FallbackEvidenceAgreementView> = Vec::new();
    for item in items.iter() {
        views.push(to_fallback_evidence_agreement_view(item));
    }
    views
}

pub fn sum_fallback_evidence_agreement_view(view: &FallbackEvidenceAgreementView) -> Result<f64, String> {
    let parsed_window_days = view
        .window_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    Ok(parsed_window_days as f64 + parsed_budget + parsed_maximum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_fallback_evidence_agreement("code-55", "title-797");
        let view = to_fallback_evidence_agreement_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_fallback_evidence_agreement_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
