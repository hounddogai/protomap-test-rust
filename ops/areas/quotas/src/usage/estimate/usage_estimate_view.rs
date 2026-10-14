use super::*;

use acme_kit::text;

/// How screens and exports show a usage estimate.
#[derive(Debug, Clone, PartialEq)]
pub struct UsageEstimateView {
    pub code_text: String,
    pub title_text: String,
    pub archived_text: String,
    pub threshold_text: String,
    pub external_ref: String,
    pub maximum: String,
    pub category: String,
    pub status_label: String,
}

pub fn to_usage_estimate_view(item: &UsageEstimate) -> UsageEstimateView {
    UsageEstimateView {
        code_text: item.code.clone(),
        title_text: item.title.clone(),
        archived_text: format!("{}", item.archived),
        threshold_text: format!("{:.2}", item.threshold),
        external_ref: text::truncate(&item.external_ref, 64),
        maximum: format!("{:.2}", item.maximum),
        category: text::truncate(&item.category, 80),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_usage_estimate_views(items: &[UsageEstimate]) -> Vec<UsageEstimateView> {
    let mut views: Vec<UsageEstimateView> = Vec::new();
    for item in items.iter() {
        views.push(to_usage_estimate_view(item));
    }
    views
}

pub fn sum_usage_estimate_view(view: &UsageEstimateView) -> Result<f64, String> {
    let parsed_threshold = view
        .threshold_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    Ok(parsed_threshold + parsed_maximum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_usage_estimate("code-466", "title-329");
        let view = to_usage_estimate_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_usage_estimate_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
