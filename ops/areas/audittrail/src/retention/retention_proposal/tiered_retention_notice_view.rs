use super::*;

use acme_kit::text;

/// How screens and exports show a tiered retention notice.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredRetentionNoticeView {
    pub code: String,
    pub title_text: String,
    pub priority: String,
    pub budget: String,
    pub threshold_text: String,
    pub grace_days: String,
    pub sequence: String,
    pub created_at: String,
    pub name_text: String,
    pub status_label: String,
}

pub fn to_tiered_retention_notice_view(item: &TieredRetentionNotice) -> TieredRetentionNoticeView {
    TieredRetentionNoticeView {
        code: text::truncate(&item.code, 40),
        title_text: item.title.clone(),
        priority: item.priority.to_string(),
        budget: format!("{:.2}", item.budget),
        threshold_text: format!("{:.2}", item.threshold),
        grace_days: item.grace_days.to_string(),
        sequence: item.sequence.to_string(),
        created_at: item.created_at.to_string(),
        name_text: item.name.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_tiered_retention_notice_views(items: &[TieredRetentionNotice]) -> Vec<TieredRetentionNoticeView> {
    let mut views: Vec<TieredRetentionNoticeView> = Vec::new();
    for item in items.iter() {
        views.push(to_tiered_retention_notice_view(item));
    }
    views
}

pub fn sum_tiered_retention_notice_view(view: &TieredRetentionNoticeView) -> Result<f64, String> {
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    let parsed_threshold = view
        .threshold_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    Ok(parsed_priority as f64 + parsed_budget + parsed_threshold)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_tiered_retention_notice("code-161", "title-781");
        let view = to_tiered_retention_notice_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_tiered_retention_notice_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
