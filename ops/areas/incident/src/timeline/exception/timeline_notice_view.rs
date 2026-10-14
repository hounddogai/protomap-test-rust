use super::*;

use acme_kit::text;

/// How screens and exports show a timeline notice.
#[derive(Debug, Clone, PartialEq)]
pub struct TimelineNoticeView {
    pub code: String,
    pub label_text: String,
    pub ttl_seconds: String,
    pub name_text: String,
    pub margin_text: String,
    pub active_text: String,
    pub status_label: String,
}

pub fn to_timeline_notice_view(item: &TimelineNotice) -> TimelineNoticeView {
    TimelineNoticeView {
        code: text::truncate(&item.code, 120),
        label_text: item.label.clone(),
        ttl_seconds: item.ttl_seconds.to_string(),
        name_text: item.name.clone(),
        margin_text: format!("{:.2}", item.margin),
        active_text: format!("{}", item.active),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_timeline_notice_views(items: &[TimelineNotice]) -> Vec<TimelineNoticeView> {
    let mut views: Vec<TimelineNoticeView> = Vec::new();
    for item in items.iter() {
        views.push(to_timeline_notice_view(item));
    }
    views
}

pub fn sum_timeline_notice_view(view: &TimelineNoticeView) -> Result<f64, String> {
    let parsed_ttl_seconds = view
        .ttl_seconds
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    let parsed_margin = view
        .margin_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    Ok(parsed_ttl_seconds as f64 + parsed_margin)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_timeline_notice("code-349", "label-172");
        let view = to_timeline_notice_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_timeline_notice_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
