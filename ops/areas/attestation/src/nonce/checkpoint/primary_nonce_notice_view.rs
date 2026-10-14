use super::*;

use acme_kit::text;

/// How screens and exports show a primary nonce notice.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryNonceNoticeView {
    pub code: String,
    pub label: String,
    pub region: String,
    pub description: String,
    pub priority: String,
    pub version_text: String,
    pub currency_text: String,
    pub archived_text: String,
    pub title: String,
    pub threshold: String,
    pub status_label: String,
}

pub fn to_primary_nonce_notice_view(item: &PrimaryNonceNotice) -> PrimaryNonceNoticeView {
    PrimaryNonceNoticeView {
        code: text::truncate(&item.code, 80),
        label: item.label.clone(),
        region: text::truncate(&item.region, 64),
        description: item.description.clone(),
        priority: item.priority.to_string(),
        version_text: item.version.to_string(),
        currency_text: item.currency.clone(),
        archived_text: format!("{}", item.archived),
        title: item.title.clone(),
        threshold: format!("{:.2}", item.threshold),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_primary_nonce_notice_views(items: &[PrimaryNonceNotice]) -> Vec<PrimaryNonceNoticeView> {
    let mut views: Vec<PrimaryNonceNoticeView> = Vec::new();
    for item in items.iter() {
        views.push(to_primary_nonce_notice_view(item));
    }
    views
}

pub fn sum_primary_nonce_notice_view(view: &PrimaryNonceNoticeView) -> Result<f64, String> {
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    let parsed_version = view
        .version_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    let parsed_threshold = view
        .threshold
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    Ok(parsed_priority as f64 + (parsed_version as f64) + parsed_threshold)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_primary_nonce_notice("code-66", "label-475");
        let view = to_primary_nonce_notice_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_primary_nonce_notice_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
