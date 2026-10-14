use super::*;

use acme_kit::text;

/// How screens and exports show a trust domain bundle.
#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainBundleView {
    pub code: String,
    pub label_text: String,
    pub phone_text: String,
    pub visible: String,
    pub rotation_days: String,
    pub name: String,
    pub batch_size: String,
    pub ends_at: String,
    pub status_label: String,
}

pub fn to_trust_domain_bundle_view(item: &TrustDomainBundle) -> TrustDomainBundleView {
    TrustDomainBundleView {
        code: text::truncate(&item.code, 64),
        label_text: item.label.clone(),
        phone_text: item.phone.clone(),
        visible: format!("{}", item.visible),
        rotation_days: item.rotation_days.to_string(),
        name: item.name.clone(),
        batch_size: item.batch_size.to_string(),
        ends_at: item.ends_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_trust_domain_bundle_views(items: &[TrustDomainBundle]) -> Vec<TrustDomainBundleView> {
    let mut views: Vec<TrustDomainBundleView> = Vec::new();
    for item in items.iter() {
        views.push(to_trust_domain_bundle_view(item));
    }
    views
}

pub fn sum_trust_domain_bundle_view(view: &TrustDomainBundleView) -> Result<f64, String> {
    let parsed_rotation_days = view
        .rotation_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    let parsed_batch_size = view
        .batch_size
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    let parsed_ends_at = view
        .ends_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    Ok(parsed_rotation_days as f64 + (parsed_batch_size as f64) + (parsed_ends_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_trust_domain_bundle("code-843", "label-986");
        let view = to_trust_domain_bundle_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_trust_domain_bundle_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
