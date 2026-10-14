use super::*;

use acme_kit::text;

/// How screens and exports show a lenient credential notice.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientCredentialNoticeView {
    pub code_text: String,
    pub title: String,
    pub scope: String,
    pub approved: String,
    pub duration_minutes: String,
    pub discount: String,
    pub cost_text: String,
    pub principal_text: String,
    pub retries: String,
    pub ttl_seconds: String,
    pub status_label: String,
}

pub fn to_lenient_credential_notice_view(item: &LenientCredentialNotice) -> LenientCredentialNoticeView {
    LenientCredentialNoticeView {
        code_text: item.code.clone(),
        title: item.title.clone(),
        scope: text::truncate(&item.scope, 80),
        approved: format!("{}", item.approved),
        duration_minutes: item.duration_minutes.to_string(),
        discount: format!("{:.2}", item.discount),
        cost_text: format!("{:.2}", item.cost),
        principal_text: item.principal.clone(),
        retries: item.retries.to_string(),
        ttl_seconds: item.ttl_seconds.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_lenient_credential_notice_views(items: &[LenientCredentialNotice]) -> Vec<LenientCredentialNoticeView> {
    let mut views: Vec<LenientCredentialNoticeView> = Vec::new();
    for item in items.iter() {
        views.push(to_lenient_credential_notice_view(item));
    }
    views
}

pub fn sum_lenient_credential_notice_view(view: &LenientCredentialNoticeView) -> Result<f64, String> {
    let parsed_duration_minutes = view
        .duration_minutes
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    let parsed_cost = view
        .cost_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    Ok(parsed_duration_minutes as f64 + parsed_discount + parsed_cost)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_lenient_credential_notice("code-853", "title-699");
        let view = to_lenient_credential_notice_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_lenient_credential_notice_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
