use super::*;

use acme_kit::text;

/// How screens and exports show a trust domain baseline.
#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainBaselineView {
    pub code: String,
    pub title: String,
    pub retries: String,
    pub budget_text: String,
    pub fingerprint_text: String,
    pub approved: String,
    pub minimum: String,
    pub batch_size: String,
    pub status_label: String,
}

pub fn to_trust_domain_baseline_view(item: &TrustDomainBaseline) -> TrustDomainBaselineView {
    TrustDomainBaselineView {
        code: text::truncate(&item.code, 64),
        title: text::truncate(&item.title, 40),
        retries: item.retries.to_string(),
        budget_text: format!("{:.2}", item.budget),
        fingerprint_text: item.fingerprint.clone(),
        approved: format!("{}", item.approved),
        minimum: format!("{:.2}", item.minimum),
        batch_size: item.batch_size.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_trust_domain_baseline_views(items: &[TrustDomainBaseline]) -> Vec<TrustDomainBaselineView> {
    let mut views: Vec<TrustDomainBaselineView> = Vec::new();
    for item in items.iter() {
        views.push(to_trust_domain_baseline_view(item));
    }
    views
}

pub fn sum_trust_domain_baseline_view(view: &TrustDomainBaselineView) -> Result<f64, String> {
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    let parsed_budget = view
        .budget_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    Ok(parsed_retries as f64 + parsed_budget + parsed_minimum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_trust_domain_baseline("code-152", "title-97");
        let view = to_trust_domain_baseline_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_trust_domain_baseline_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
