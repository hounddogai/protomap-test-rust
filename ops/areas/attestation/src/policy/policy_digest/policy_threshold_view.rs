use super::*;

use acme_kit::text;

/// How screens and exports show a policy threshold.
#[derive(Debug, Clone, PartialEq)]
pub struct PolicyThresholdView {
    pub code: String,
    pub label: String,
    pub version_text: String,
    pub segment_text: String,
    pub minimum: String,
    pub maximum: String,
    pub expires_at: String,
    pub country: String,
    pub visible: String,
    pub status_label: String,
}

pub fn to_policy_threshold_view(item: &PolicyThreshold) -> PolicyThresholdView {
    PolicyThresholdView {
        code: text::truncate(&item.code, 40),
        label: text::truncate(&item.label, 80),
        version_text: item.version.to_string(),
        segment_text: text::truncate(&item.segment, 120),
        minimum: format!("{:.2}", item.minimum),
        maximum: format!("{:.2}", item.maximum),
        expires_at: item.expires_at.to_string(),
        country: text::truncate(&item.country, 40),
        visible: format!("{}", item.visible),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_policy_threshold_views(items: &[PolicyThreshold]) -> Vec<PolicyThresholdView> {
    let mut views: Vec<PolicyThresholdView> = Vec::new();
    for item in items.iter() {
        views.push(to_policy_threshold_view(item));
    }
    views
}

pub fn sum_policy_threshold_view(view: &PolicyThresholdView) -> Result<f64, String> {
    let parsed_version = view
        .version_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    Ok(parsed_version as f64 + parsed_minimum + parsed_maximum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_policy_threshold("code-726", "label-36");
        let view = to_policy_threshold_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_policy_threshold_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
