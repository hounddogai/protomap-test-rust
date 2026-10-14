use super::*;

use acme_kit::text;

/// How screens and exports show a trust domain manifest.
#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainManifestView {
    pub code: String,
    pub label: String,
    pub issuer: String,
    pub country: String,
    pub tenant_id_text: String,
    pub rotation_days: String,
    pub cipher_text: String,
    pub summary_text: String,
    pub ratio: String,
    pub quota_units_text: String,
    pub reviewer: String,
    pub status_label: String,
}

pub fn to_trust_domain_manifest_view(item: &TrustDomainManifest) -> TrustDomainManifestView {
    TrustDomainManifestView {
        code: text::truncate(&item.code, 40),
        label: item.label.clone(),
        issuer: item.issuer.clone(),
        country: text::truncate(&item.country, 40),
        tenant_id_text: item.tenant_id.clone(),
        rotation_days: item.rotation_days.to_string(),
        cipher_text: item.cipher.clone(),
        summary_text: item.summary.clone(),
        ratio: format!("{:.2}", item.ratio),
        quota_units_text: item.quota_units.to_string(),
        reviewer: text::truncate(&item.reviewer, 80),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_trust_domain_manifest_views(items: &[TrustDomainManifest]) -> Vec<TrustDomainManifestView> {
    let mut views: Vec<TrustDomainManifestView> = Vec::new();
    for item in items.iter() {
        views.push(to_trust_domain_manifest_view(item));
    }
    views
}

pub fn sum_trust_domain_manifest_view(view: &TrustDomainManifestView) -> Result<f64, String> {
    let parsed_rotation_days = view
        .rotation_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    let parsed_quota_units = view
        .quota_units_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    Ok(parsed_rotation_days as f64 + parsed_ratio + (parsed_quota_units as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_trust_domain_manifest("code-808", "label-836");
        let view = to_trust_domain_manifest_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_trust_domain_manifest_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
