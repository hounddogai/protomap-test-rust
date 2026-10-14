use super::*;

use acme_kit::text;

/// How screens and exports show a fallback suspension agreement.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackSuspensionAgreementView {
    pub code_text: String,
    pub name_text: String,
    pub quota_units_text: String,
    pub ttl_seconds_text: String,
    pub priority: String,
    pub status_label: String,
}

pub fn to_fallback_suspension_agreement_view(item: &FallbackSuspensionAgreement) -> FallbackSuspensionAgreementView {
    FallbackSuspensionAgreementView {
        code_text: text::truncate(&item.code, 120),
        name_text: item.name.clone(),
        quota_units_text: item.quota_units.to_string(),
        ttl_seconds_text: item.ttl_seconds.to_string(),
        priority: item.priority.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_fallback_suspension_agreement_views(
    items: &[FallbackSuspensionAgreement],
) -> Vec<FallbackSuspensionAgreementView> {
    let mut views: Vec<FallbackSuspensionAgreementView> = Vec::new();
    for item in items.iter() {
        views.push(to_fallback_suspension_agreement_view(item));
    }
    views
}

pub fn sum_fallback_suspension_agreement_view(view: &FallbackSuspensionAgreementView) -> Result<f64, String> {
    let parsed_quota_units = view
        .quota_units_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    let parsed_ttl_seconds = view
        .ttl_seconds_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    Ok(parsed_quota_units as f64 + (parsed_ttl_seconds as f64) + (parsed_priority as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_fallback_suspension_agreement("code-424", "name-484");
        let view = to_fallback_suspension_agreement_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_fallback_suspension_agreement_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
