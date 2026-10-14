use super::*;

use acme_kit::text;

/// How screens and exports show a fallback identity quota.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackIdentityQuotaView {
    pub code_text: String,
    pub name_text: String,
    pub priority: String,
    pub score: String,
    pub discount_text: String,
    pub ends_at_text: String,
    pub region_text: String,
    pub ttl_seconds: String,
    pub budget: String,
    pub attempts_text: String,
    pub status_label: String,
}

pub fn to_fallback_identity_quota_view(item: &FallbackIdentityQuota) -> FallbackIdentityQuotaView {
    FallbackIdentityQuotaView {
        code_text: item.code.clone(),
        name_text: item.name.clone(),
        priority: item.priority.to_string(),
        score: format!("{:.2}", item.score),
        discount_text: format!("{:.2}", item.discount),
        ends_at_text: item.ends_at.to_string(),
        region_text: text::truncate(&item.region, 80),
        ttl_seconds: item.ttl_seconds.to_string(),
        budget: format!("{:.2}", item.budget),
        attempts_text: item.attempts.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_fallback_identity_quota_views(items: &[FallbackIdentityQuota]) -> Vec<FallbackIdentityQuotaView> {
    let mut views: Vec<FallbackIdentityQuotaView> = Vec::new();
    for item in items.iter() {
        views.push(to_fallback_identity_quota_view(item));
    }
    views
}

pub fn sum_fallback_identity_quota_view(view: &FallbackIdentityQuotaView) -> Result<f64, String> {
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    let parsed_discount = view
        .discount_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    Ok(parsed_priority as f64 + parsed_score + parsed_discount)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_fallback_identity_quota("code-417", "name-700");
        let view = to_fallback_identity_quota_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_fallback_identity_quota_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
