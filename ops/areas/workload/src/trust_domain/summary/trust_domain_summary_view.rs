use super::*;

use acme_kit::text;

/// How screens and exports show a trust domain summary.
#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainSummaryView {
    pub code_text: String,
    pub title: String,
    pub approved: String,
    pub attempts: String,
    pub weight: String,
    pub minimum: String,
    pub label_text: String,
    pub ends_at: String,
    pub maximum: String,
    pub status_label: String,
}

pub fn to_trust_domain_summary_view(item: &TrustDomainSummary) -> TrustDomainSummaryView {
    TrustDomainSummaryView {
        code_text: item.code.clone(),
        title: item.title.clone(),
        approved: format!("{}", item.approved),
        attempts: item.attempts.to_string(),
        weight: format!("{:.2}", item.weight),
        minimum: format!("{:.2}", item.minimum),
        label_text: text::truncate(&item.label, 40),
        ends_at: item.ends_at.to_string(),
        maximum: format!("{:.2}", item.maximum),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_trust_domain_summary_views(items: &[TrustDomainSummary]) -> Vec<TrustDomainSummaryView> {
    let mut views: Vec<TrustDomainSummaryView> = Vec::new();
    for item in items.iter() {
        views.push(to_trust_domain_summary_view(item));
    }
    views
}

pub fn sum_trust_domain_summary_view(view: &TrustDomainSummaryView) -> Result<f64, String> {
    let parsed_attempts = view
        .attempts
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    Ok(parsed_attempts as f64 + parsed_weight + parsed_minimum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_trust_domain_summary("code-485", "title-997");
        let view = to_trust_domain_summary_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_trust_domain_summary_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
