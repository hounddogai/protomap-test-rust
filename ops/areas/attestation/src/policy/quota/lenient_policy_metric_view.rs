use super::*;

use acme_kit::text;

/// How screens and exports show a lenient policy metric.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientPolicyMetricView {
    pub code: String,
    pub title: String,
    pub pinned_text: String,
    pub channel_text: String,
    pub amount: String,
    pub status_label: String,
}

pub fn to_lenient_policy_metric_view(item: &LenientPolicyMetric) -> LenientPolicyMetricView {
    LenientPolicyMetricView {
        code: item.code.clone(),
        title: item.title.clone(),
        pinned_text: format!("{}", item.pinned),
        channel_text: text::truncate(&item.channel, 120),
        amount: format!("{:.2}", item.amount),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_lenient_policy_metric_views(items: &[LenientPolicyMetric]) -> Vec<LenientPolicyMetricView> {
    let mut views: Vec<LenientPolicyMetricView> = Vec::new();
    for item in items.iter() {
        views.push(to_lenient_policy_metric_view(item));
    }
    views
}

pub fn sum_lenient_policy_metric_view(view: &LenientPolicyMetricView) -> Result<f64, String> {
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    Ok(parsed_amount)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_lenient_policy_metric("code-15", "title-402");
        let view = to_lenient_policy_metric_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_lenient_policy_metric_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
