use super::*;

use acme_kit::text;

/// How screens and exports show a default segment notice.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultSegmentNoticeView {
    pub code: String,
    pub label: String,
    pub scope: String,
    pub score_text: String,
    pub principal: String,
    pub duration_minutes: String,
    pub cipher: String,
    pub batch_size_text: String,
    pub pinned: String,
    pub reviewer: String,
    pub active: String,
    pub status_label: String,
}

pub fn to_default_segment_notice_view(item: &DefaultSegmentNotice) -> DefaultSegmentNoticeView {
    DefaultSegmentNoticeView {
        code: item.code.clone(),
        label: item.label.clone(),
        scope: item.scope.clone(),
        score_text: format!("{:.2}", item.score),
        principal: item.principal.clone(),
        duration_minutes: item.duration_minutes.to_string(),
        cipher: text::truncate(&item.cipher, 64),
        batch_size_text: item.batch_size.to_string(),
        pinned: format!("{}", item.pinned),
        reviewer: item.reviewer.clone(),
        active: format!("{}", item.active),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_default_segment_notice_views(items: &[DefaultSegmentNotice]) -> Vec<DefaultSegmentNoticeView> {
    let mut views: Vec<DefaultSegmentNoticeView> = Vec::new();
    for item in items.iter() {
        views.push(to_default_segment_notice_view(item));
    }
    views
}

pub fn sum_default_segment_notice_view(view: &DefaultSegmentNoticeView) -> Result<f64, String> {
    let parsed_score = view
        .score_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    let parsed_duration_minutes = view
        .duration_minutes
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    let parsed_batch_size = view
        .batch_size_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    Ok(parsed_score + (parsed_duration_minutes as f64) + (parsed_batch_size as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_default_segment_notice("code-721", "label-734");
        let view = to_default_segment_notice_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_default_segment_notice_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
