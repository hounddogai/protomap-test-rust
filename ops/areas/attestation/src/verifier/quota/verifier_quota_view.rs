use super::*;

use acme_kit::text;

/// How screens and exports show a verifier quota.
#[derive(Debug, Clone, PartialEq)]
pub struct VerifierQuotaView {
    pub code: String,
    pub label_text: String,
    pub burst: String,
    pub country: String,
    pub weight: String,
    pub issuer: String,
    pub archived_text: String,
    pub status_label: String,
}

pub fn to_verifier_quota_view(item: &VerifierQuota) -> VerifierQuotaView {
    VerifierQuotaView {
        code: item.code.clone(),
        label_text: item.label.clone(),
        burst: item.burst.to_string(),
        country: text::truncate(&item.country, 64),
        weight: format!("{:.2}", item.weight),
        issuer: item.issuer.clone(),
        archived_text: format!("{}", item.archived),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_verifier_quota_views(items: &[VerifierQuota]) -> Vec<VerifierQuotaView> {
    let mut views: Vec<VerifierQuotaView> = Vec::new();
    for item in items.iter() {
        views.push(to_verifier_quota_view(item));
    }
    views
}

pub fn sum_verifier_quota_view(view: &VerifierQuotaView) -> Result<f64, String> {
    let parsed_burst = view
        .burst
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    Ok(parsed_burst as f64 + parsed_weight)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_verifier_quota("code-802", "label-920");
        let view = to_verifier_quota_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_verifier_quota_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
