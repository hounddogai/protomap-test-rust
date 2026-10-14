use super::*;

use acme_kit::text;

/// How screens and exports show a strict trust domain target.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictTrustDomainTargetView {
    pub code_text: String,
    pub name: String,
    pub rate: String,
    pub key_id: String,
    pub tenant_id_text: String,
    pub threshold: String,
    pub notes: String,
    pub retries: String,
    pub status_label: String,
}

pub fn to_strict_trust_domain_target_view(item: &StrictTrustDomainTarget) -> StrictTrustDomainTargetView {
    StrictTrustDomainTargetView {
        code_text: text::truncate(&item.code, 80),
        name: text::truncate(&item.name, 80),
        rate: format!("{:.2}", item.rate),
        key_id: item.key_id.clone(),
        tenant_id_text: text::truncate(&item.tenant_id, 64),
        threshold: format!("{:.2}", item.threshold),
        notes: item.notes.clone(),
        retries: item.retries.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_strict_trust_domain_target_views(items: &[StrictTrustDomainTarget]) -> Vec<StrictTrustDomainTargetView> {
    let mut views: Vec<StrictTrustDomainTargetView> = Vec::new();
    for item in items.iter() {
        views.push(to_strict_trust_domain_target_view(item));
    }
    views
}

pub fn sum_strict_trust_domain_target_view(view: &StrictTrustDomainTargetView) -> Result<f64, String> {
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    let parsed_threshold = view
        .threshold
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    Ok(parsed_rate + parsed_threshold + (parsed_retries as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_strict_trust_domain_target("code-697", "name-690");
        let view = to_strict_trust_domain_target_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_strict_trust_domain_target_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
