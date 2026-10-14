use super::*;

/// How screens and exports show a rolling runbook bundle.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingRunbookBundleView {
    pub code: String,
    pub label: String,
    pub ttl_seconds: String,
    pub segment: String,
    pub host: String,
    pub approved: String,
    pub audience_text: String,
    pub cipher: String,
    pub pinned: String,
    pub maximum: String,
    pub status_label: String,
}

pub fn to_rolling_runbook_bundle_view(item: &RollingRunbookBundle) -> RollingRunbookBundleView {
    RollingRunbookBundleView {
        code: item.code.clone(),
        label: item.label.clone(),
        ttl_seconds: item.ttl_seconds.to_string(),
        segment: item.segment.clone(),
        host: item.host.clone(),
        approved: format!("{}", item.approved),
        audience_text: item.audience.clone(),
        cipher: item.cipher.clone(),
        pinned: format!("{}", item.pinned),
        maximum: format!("{:.2}", item.maximum),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rolling_runbook_bundle_views(items: &[RollingRunbookBundle]) -> Vec<RollingRunbookBundleView> {
    let mut views: Vec<RollingRunbookBundleView> = Vec::new();
    for item in items.iter() {
        views.push(to_rolling_runbook_bundle_view(item));
    }
    views
}

pub fn sum_rolling_runbook_bundle_view(view: &RollingRunbookBundleView) -> Result<f64, String> {
    let parsed_ttl_seconds = view
        .ttl_seconds
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    Ok(parsed_ttl_seconds as f64 + parsed_maximum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_rolling_runbook_bundle("code-946", "label-967");
        let view = to_rolling_runbook_bundle_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_rolling_runbook_bundle_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
