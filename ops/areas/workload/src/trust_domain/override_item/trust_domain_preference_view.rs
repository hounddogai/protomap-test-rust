use super::*;

use acme_kit::text;

/// How screens and exports show a trust domain preference.
#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainPreferenceView {
    pub code: String,
    pub name: String,
    pub ratio: String,
    pub cluster_text: String,
    pub priority: String,
    pub unit: String,
    pub issuer_text: String,
    pub owner_id: String,
    pub key_id_text: String,
    pub visible: String,
    pub status_label: String,
}

pub fn to_trust_domain_preference_view(item: &TrustDomainPreference) -> TrustDomainPreferenceView {
    TrustDomainPreferenceView {
        code: text::truncate(&item.code, 64),
        name: text::truncate(&item.name, 64),
        ratio: format!("{:.2}", item.ratio),
        cluster_text: item.cluster.clone(),
        priority: item.priority.to_string(),
        unit: item.unit.clone(),
        issuer_text: item.issuer.clone(),
        owner_id: text::truncate(&item.owner_id, 120),
        key_id_text: item.key_id.clone(),
        visible: format!("{}", item.visible),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_trust_domain_preference_views(items: &[TrustDomainPreference]) -> Vec<TrustDomainPreferenceView> {
    let mut views: Vec<TrustDomainPreferenceView> = Vec::new();
    for item in items.iter() {
        views.push(to_trust_domain_preference_view(item));
    }
    views
}

pub fn sum_trust_domain_preference_view(view: &TrustDomainPreferenceView) -> Result<f64, String> {
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    Ok(parsed_ratio + (parsed_priority as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_trust_domain_preference("code-297", "name-331");
        let view = to_trust_domain_preference_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_trust_domain_preference_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
