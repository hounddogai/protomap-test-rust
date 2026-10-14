use super::*;

use acme_kit::text;

/// How screens and exports show a approval notice.
#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalNoticeView {
    pub code: String,
    pub label: String,
    pub title: String,
    pub retries: String,
    pub reviewer: String,
    pub external_ref: String,
    pub threshold: String,
    pub batch_size_text: String,
    pub starts_at_text: String,
    pub status_label: String,
}

pub fn to_approval_notice_view(item: &ApprovalNotice) -> ApprovalNoticeView {
    ApprovalNoticeView {
        code: item.code.clone(),
        label: item.label.clone(),
        title: item.title.clone(),
        retries: item.retries.to_string(),
        reviewer: text::truncate(&item.reviewer, 64),
        external_ref: text::truncate(&item.external_ref, 120),
        threshold: format!("{:.2}", item.threshold),
        batch_size_text: item.batch_size.to_string(),
        starts_at_text: item.starts_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_approval_notice_views(items: &[ApprovalNotice]) -> Vec<ApprovalNoticeView> {
    let mut views: Vec<ApprovalNoticeView> = Vec::new();
    for item in items.iter() {
        views.push(to_approval_notice_view(item));
    }
    views
}

pub fn sum_approval_notice_view(view: &ApprovalNoticeView) -> Result<f64, String> {
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    let parsed_threshold = view
        .threshold
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    let parsed_batch_size = view
        .batch_size_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    Ok(parsed_retries as f64 + parsed_threshold + (parsed_batch_size as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_approval_notice("code-515", "label-653");
        let view = to_approval_notice_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_approval_notice_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
