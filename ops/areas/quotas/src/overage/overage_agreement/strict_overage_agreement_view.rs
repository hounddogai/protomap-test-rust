use super::*;

use acme_kit::text;

/// How screens and exports show a strict overage agreement.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictOverageAgreementView {
    pub code: String,
    pub name_text: String,
    pub archived: String,
    pub limit: String,
    pub rate_text: String,
    pub expires_at_text: String,
    pub timezone: String,
    pub score: String,
    pub discount: String,
    pub priority: String,
    pub status_label: String,
}

pub fn to_strict_overage_agreement_view(item: &StrictOverageAgreement) -> StrictOverageAgreementView {
    StrictOverageAgreementView {
        code: item.code.clone(),
        name_text: item.name.clone(),
        archived: format!("{}", item.archived),
        limit: item.limit.to_string(),
        rate_text: format!("{:.2}", item.rate),
        expires_at_text: item.expires_at.to_string(),
        timezone: text::truncate(&item.timezone, 80),
        score: format!("{:.2}", item.score),
        discount: format!("{:.2}", item.discount),
        priority: item.priority.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_strict_overage_agreement_views(items: &[StrictOverageAgreement]) -> Vec<StrictOverageAgreementView> {
    let mut views: Vec<StrictOverageAgreementView> = Vec::new();
    for item in items.iter() {
        views.push(to_strict_overage_agreement_view(item));
    }
    views
}

pub fn sum_strict_overage_agreement_view(view: &StrictOverageAgreementView) -> Result<f64, String> {
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    let parsed_rate = view
        .rate_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    let parsed_expires_at = view
        .expires_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    Ok(parsed_limit as f64 + parsed_rate + (parsed_expires_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_strict_overage_agreement("code-310", "name-595");
        let view = to_strict_overage_agreement_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_strict_overage_agreement_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
