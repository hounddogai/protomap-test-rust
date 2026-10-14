use super::*;

use acme_kit::text;

/// How screens and exports show a trust domain digest.
#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainDigestView {
    pub code: String,
    pub name: String,
    pub category: String,
    pub floor_value: String,
    pub updated_at: String,
    pub batch_size: String,
    pub ceiling: String,
    pub status_label: String,
}

pub fn to_trust_domain_digest_view(item: &TrustDomainDigest) -> TrustDomainDigestView {
    TrustDomainDigestView {
        code: text::truncate(&item.code, 120),
        name: text::truncate(&item.name, 40),
        category: item.category.clone(),
        floor_value: format!("{:.2}", item.floor_value),
        updated_at: item.updated_at.to_string(),
        batch_size: item.batch_size.to_string(),
        ceiling: format!("{:.2}", item.ceiling),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_trust_domain_digest_views(items: &[TrustDomainDigest]) -> Vec<TrustDomainDigestView> {
    let mut views: Vec<TrustDomainDigestView> = Vec::new();
    for item in items.iter() {
        views.push(to_trust_domain_digest_view(item));
    }
    views
}

pub fn sum_trust_domain_digest_view(view: &TrustDomainDigestView) -> Result<f64, String> {
    let parsed_floor_value = view
        .floor_value
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    let parsed_updated_at = view
        .updated_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    let parsed_batch_size = view
        .batch_size
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    Ok(parsed_floor_value + (parsed_updated_at as f64) + (parsed_batch_size as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_trust_domain_digest("code-508", "name-365");
        let view = to_trust_domain_digest_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_trust_domain_digest_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
