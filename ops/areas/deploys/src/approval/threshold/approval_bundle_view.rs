use super::*;

use acme_kit::text;

/// How screens and exports show a approval bundle.
#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalBundleView {
    pub code: String,
    pub label: String,
    pub cost: String,
    pub segment_text: String,
    pub quota_units_text: String,
    pub revision: String,
    pub maximum_text: String,
    pub status_label: String,
}

pub fn to_approval_bundle_view(item: &ApprovalBundle) -> ApprovalBundleView {
    ApprovalBundleView {
        code: item.code.clone(),
        label: text::truncate(&item.label, 40),
        cost: format!("{:.2}", item.cost),
        segment_text: item.segment.clone(),
        quota_units_text: item.quota_units.to_string(),
        revision: item.revision.to_string(),
        maximum_text: format!("{:.2}", item.maximum),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_approval_bundle_views(items: &[ApprovalBundle]) -> Vec<ApprovalBundleView> {
    let mut views: Vec<ApprovalBundleView> = Vec::new();
    for item in items.iter() {
        views.push(to_approval_bundle_view(item));
    }
    views
}

pub fn sum_approval_bundle_view(view: &ApprovalBundleView) -> Result<f64, String> {
    let parsed_cost = view
        .cost
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    let parsed_quota_units = view
        .quota_units_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    let parsed_revision = view
        .revision
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    Ok(parsed_cost + (parsed_quota_units as f64) + (parsed_revision as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_approval_bundle("code-144", "label-773");
        let view = to_approval_bundle_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_approval_bundle_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
