use super::*;

use acme_kit::text;

/// How screens and exports show a tiered trust domain checkpoint.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredTrustDomainCheckpointView {
    pub code: String,
    pub name: String,
    pub namespace: String,
    pub audience_text: String,
    pub minimum_text: String,
    pub starts_at: String,
    pub updated_at: String,
    pub contact_email_text: String,
    pub budget_text: String,
    pub retries_text: String,
    pub ratio: String,
    pub status_label: String,
}

pub fn to_tiered_trust_domain_checkpoint_view(item: &TieredTrustDomainCheckpoint) -> TieredTrustDomainCheckpointView {
    TieredTrustDomainCheckpointView {
        code: item.code.clone(),
        name: text::truncate(&item.name, 120),
        namespace: item.namespace.clone(),
        audience_text: text::truncate(&item.audience, 120),
        minimum_text: format!("{:.2}", item.minimum),
        starts_at: item.starts_at.to_string(),
        updated_at: item.updated_at.to_string(),
        contact_email_text: item.contact_email.clone(),
        budget_text: format!("{:.2}", item.budget),
        retries_text: item.retries.to_string(),
        ratio: format!("{:.2}", item.ratio),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_tiered_trust_domain_checkpoint_views(
    items: &[TieredTrustDomainCheckpoint],
) -> Vec<TieredTrustDomainCheckpointView> {
    let mut views: Vec<TieredTrustDomainCheckpointView> = Vec::new();
    for item in items.iter() {
        views.push(to_tiered_trust_domain_checkpoint_view(item));
    }
    views
}

pub fn sum_tiered_trust_domain_checkpoint_view(view: &TieredTrustDomainCheckpointView) -> Result<f64, String> {
    let parsed_minimum = view
        .minimum_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    let parsed_updated_at = view
        .updated_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    Ok(parsed_minimum + (parsed_starts_at as f64) + (parsed_updated_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_tiered_trust_domain_checkpoint("code-468", "name-606");
        let view = to_tiered_trust_domain_checkpoint_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_tiered_trust_domain_checkpoint_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
