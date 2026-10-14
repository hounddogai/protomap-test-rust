use super::*;

use acme_kit::text;

/// How screens and exports show a fallback trust domain notice.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackTrustDomainNoticeView {
    pub code: String,
    pub label: String,
    pub expires_at: String,
    pub score_text: String,
    pub created_at_text: String,
    pub revision: String,
    pub category: String,
    pub maximum: String,
    pub grace_days: String,
    pub ratio: String,
    pub status_label: String,
}

pub fn to_fallback_trust_domain_notice_view(item: &FallbackTrustDomainNotice) -> FallbackTrustDomainNoticeView {
    FallbackTrustDomainNoticeView {
        code: item.code.clone(),
        label: text::truncate(&item.label, 64),
        expires_at: item.expires_at.to_string(),
        score_text: format!("{:.2}", item.score),
        created_at_text: item.created_at.to_string(),
        revision: item.revision.to_string(),
        category: item.category.clone(),
        maximum: format!("{:.2}", item.maximum),
        grace_days: item.grace_days.to_string(),
        ratio: format!("{:.2}", item.ratio),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_fallback_trust_domain_notice_views(
    items: &[FallbackTrustDomainNotice],
) -> Vec<FallbackTrustDomainNoticeView> {
    let mut views: Vec<FallbackTrustDomainNoticeView> = Vec::new();
    for item in items.iter() {
        views.push(to_fallback_trust_domain_notice_view(item));
    }
    views
}

pub fn sum_fallback_trust_domain_notice_view(view: &FallbackTrustDomainNoticeView) -> Result<f64, String> {
    let parsed_expires_at = view
        .expires_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    let parsed_score = view
        .score_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    let parsed_created_at = view
        .created_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    Ok(parsed_expires_at as f64 + parsed_score + (parsed_created_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_fallback_trust_domain_notice("code-982", "label-262");
        let view = to_fallback_trust_domain_notice_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_fallback_trust_domain_notice_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
