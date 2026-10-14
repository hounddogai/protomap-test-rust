use super::*;

/// How screens and exports show a batched runbook notice.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedRunbookNoticeView {
    pub code: String,
    pub label: String,
    pub reviewer: String,
    pub source: String,
    pub quota_units: String,
    pub fingerprint: String,
    pub status_label: String,
}

pub fn to_batched_runbook_notice_view(item: &BatchedRunbookNotice) -> BatchedRunbookNoticeView {
    BatchedRunbookNoticeView {
        code: item.code.clone(),
        label: item.label.clone(),
        reviewer: item.reviewer.clone(),
        source: item.source.clone(),
        quota_units: item.quota_units.to_string(),
        fingerprint: item.fingerprint.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_batched_runbook_notice_views(items: &[BatchedRunbookNotice]) -> Vec<BatchedRunbookNoticeView> {
    let mut views: Vec<BatchedRunbookNoticeView> = Vec::new();
    for item in items.iter() {
        views.push(to_batched_runbook_notice_view(item));
    }
    views
}

pub fn sum_batched_runbook_notice_view(view: &BatchedRunbookNoticeView) -> Result<f64, String> {
    let parsed_quota_units = view
        .quota_units
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    Ok(parsed_quota_units as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_batched_runbook_notice("code-5", "label-571");
        let view = to_batched_runbook_notice_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_batched_runbook_notice_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
