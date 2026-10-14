use super::*;

use acme_kit::text;

/// How screens and exports show a verifier notice.
#[derive(Debug, Clone, PartialEq)]
pub struct VerifierNoticeView {
    pub code: String,
    pub name: String,
    pub locale: String,
    pub contact_email: String,
    pub host_text: String,
    pub ratio_text: String,
    pub limit_text: String,
    pub country: String,
    pub burst: String,
    pub ttl_seconds: String,
    pub status_label: String,
}

pub fn to_verifier_notice_view(item: &VerifierNotice) -> VerifierNoticeView {
    VerifierNoticeView {
        code: text::truncate(&item.code, 120),
        name: text::truncate(&item.name, 120),
        locale: item.locale.clone(),
        contact_email: item.contact_email.clone(),
        host_text: item.host.clone(),
        ratio_text: format!("{:.2}", item.ratio),
        limit_text: item.limit.to_string(),
        country: text::truncate(&item.country, 80),
        burst: item.burst.to_string(),
        ttl_seconds: item.ttl_seconds.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_verifier_notice_views(items: &[VerifierNotice]) -> Vec<VerifierNoticeView> {
    let mut views: Vec<VerifierNoticeView> = Vec::new();
    for item in items.iter() {
        views.push(to_verifier_notice_view(item));
    }
    views
}

pub fn sum_verifier_notice_view(view: &VerifierNoticeView) -> Result<f64, String> {
    let parsed_ratio = view
        .ratio_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    let parsed_limit = view
        .limit_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    let parsed_burst = view
        .burst
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    Ok(parsed_ratio + (parsed_limit as f64) + (parsed_burst as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_verifier_notice("code-244", "name-165");
        let view = to_verifier_notice_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_verifier_notice_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
