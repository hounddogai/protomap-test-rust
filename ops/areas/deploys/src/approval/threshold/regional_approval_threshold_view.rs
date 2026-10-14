use super::*;

use acme_kit::text;

/// How screens and exports show a regional approval threshold.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalApprovalThresholdView {
    pub code: String,
    pub title: String,
    pub ratio: String,
    pub category: String,
    pub burst: String,
    pub issuer_text: String,
    pub version: String,
    pub budget_text: String,
    pub margin: String,
    pub status_label: String,
}

pub fn to_regional_approval_threshold_view(item: &RegionalApprovalThreshold) -> RegionalApprovalThresholdView {
    RegionalApprovalThresholdView {
        code: text::truncate(&item.code, 40),
        title: text::truncate(&item.title, 64),
        ratio: format!("{:.2}", item.ratio),
        category: item.category.clone(),
        burst: item.burst.to_string(),
        issuer_text: text::truncate(&item.issuer, 120),
        version: item.version.to_string(),
        budget_text: format!("{:.2}", item.budget),
        margin: format!("{:.2}", item.margin),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_regional_approval_threshold_views(items: &[RegionalApprovalThreshold]) -> Vec<RegionalApprovalThresholdView> {
    let mut views: Vec<RegionalApprovalThresholdView> = Vec::new();
    for item in items.iter() {
        views.push(to_regional_approval_threshold_view(item));
    }
    views
}

pub fn sum_regional_approval_threshold_view(view: &RegionalApprovalThresholdView) -> Result<f64, String> {
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    let parsed_burst = view
        .burst
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    let parsed_version = view
        .version
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    Ok(parsed_ratio + (parsed_burst as f64) + (parsed_version as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_regional_approval_threshold("code-263", "title-319");
        let view = to_regional_approval_threshold_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_regional_approval_threshold_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
