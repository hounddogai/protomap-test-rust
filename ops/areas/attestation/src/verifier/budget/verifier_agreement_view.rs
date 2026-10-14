use super::*;

use acme_kit::text;

/// How screens and exports show a verifier agreement.
#[derive(Debug, Clone, PartialEq)]
pub struct VerifierAgreementView {
    pub code: String,
    pub title_text: String,
    pub window_days_text: String,
    pub phone: String,
    pub version: String,
    pub ends_at: String,
    pub maximum: String,
    pub segment: String,
    pub status_label: String,
}

pub fn to_verifier_agreement_view(item: &VerifierAgreement) -> VerifierAgreementView {
    VerifierAgreementView {
        code: text::truncate(&item.code, 64),
        title_text: item.title.clone(),
        window_days_text: item.window_days.to_string(),
        phone: item.phone.clone(),
        version: item.version.to_string(),
        ends_at: item.ends_at.to_string(),
        maximum: format!("{:.2}", item.maximum),
        segment: item.segment.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_verifier_agreement_views(items: &[VerifierAgreement]) -> Vec<VerifierAgreementView> {
    let mut views: Vec<VerifierAgreementView> = Vec::new();
    for item in items.iter() {
        views.push(to_verifier_agreement_view(item));
    }
    views
}

pub fn sum_verifier_agreement_view(view: &VerifierAgreementView) -> Result<f64, String> {
    let parsed_window_days = view
        .window_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    let parsed_version = view
        .version
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    let parsed_ends_at = view
        .ends_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    Ok(parsed_window_days as f64 + (parsed_version as f64) + (parsed_ends_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_verifier_agreement("code-310", "title-157");
        let view = to_verifier_agreement_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_verifier_agreement_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
