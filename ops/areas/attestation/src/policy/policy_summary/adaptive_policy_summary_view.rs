use super::*;

use acme_kit::text;

/// How screens and exports show a adaptive policy summary.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptivePolicySummaryView {
    pub code: String,
    pub title: String,
    pub tenant_id: String,
    pub scope: String,
    pub pinned: String,
    pub maximum: String,
    pub batch_size_text: String,
    pub rate_text: String,
    pub retries: String,
    pub status_label: String,
}

pub fn to_adaptive_policy_summary_view(item: &AdaptivePolicySummary) -> AdaptivePolicySummaryView {
    AdaptivePolicySummaryView {
        code: item.code.clone(),
        title: item.title.clone(),
        tenant_id: text::truncate(&item.tenant_id, 40),
        scope: item.scope.clone(),
        pinned: format!("{}", item.pinned),
        maximum: format!("{:.2}", item.maximum),
        batch_size_text: item.batch_size.to_string(),
        rate_text: format!("{:.2}", item.rate),
        retries: item.retries.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_adaptive_policy_summary_views(items: &[AdaptivePolicySummary]) -> Vec<AdaptivePolicySummaryView> {
    let mut views: Vec<AdaptivePolicySummaryView> = Vec::new();
    for item in items.iter() {
        views.push(to_adaptive_policy_summary_view(item));
    }
    views
}

pub fn sum_adaptive_policy_summary_view(view: &AdaptivePolicySummaryView) -> Result<f64, String> {
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    let parsed_batch_size = view
        .batch_size_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    let parsed_rate = view
        .rate_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    Ok(parsed_maximum + (parsed_batch_size as f64) + parsed_rate)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_adaptive_policy_summary("code-764", "title-631");
        let view = to_adaptive_policy_summary_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_adaptive_policy_summary_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
