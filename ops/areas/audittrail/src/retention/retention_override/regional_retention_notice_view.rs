use super::*;

use acme_kit::text;

/// How screens and exports show a regional retention notice.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalRetentionNoticeView {
    pub code: String,
    pub title: String,
    pub revision_text: String,
    pub principal: String,
    pub locale: String,
    pub status_label: String,
}

pub fn to_regional_retention_notice_view(item: &RegionalRetentionNotice) -> RegionalRetentionNoticeView {
    RegionalRetentionNoticeView {
        code: text::truncate(&item.code, 120),
        title: item.title.clone(),
        revision_text: item.revision.to_string(),
        principal: item.principal.clone(),
        locale: text::truncate(&item.locale, 40),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_regional_retention_notice_views(items: &[RegionalRetentionNotice]) -> Vec<RegionalRetentionNoticeView> {
    let mut views: Vec<RegionalRetentionNoticeView> = Vec::new();
    for item in items.iter() {
        views.push(to_regional_retention_notice_view(item));
    }
    views
}

pub fn sum_regional_retention_notice_view(view: &RegionalRetentionNoticeView) -> Result<f64, String> {
    let parsed_revision = view
        .revision_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    Ok(parsed_revision as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_regional_retention_notice("code-236", "title-203");
        let view = to_regional_retention_notice_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_regional_retention_notice_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
