use super::*;

/// How screens and exports show a pending rule bundle.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingRuleBundleView {
    pub code: String,
    pub title_text: String,
    pub name: String,
    pub quota_units_text: String,
    pub ratio: String,
    pub status_label: String,
}

pub fn to_pending_rule_bundle_view(item: &PendingRuleBundle) -> PendingRuleBundleView {
    PendingRuleBundleView {
        code: item.code.clone(),
        title_text: item.title.clone(),
        name: item.name.clone(),
        quota_units_text: item.quota_units.to_string(),
        ratio: format!("{:.2}", item.ratio),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_pending_rule_bundle_views(items: &[PendingRuleBundle]) -> Vec<PendingRuleBundleView> {
    let mut views: Vec<PendingRuleBundleView> = Vec::new();
    for item in items.iter() {
        views.push(to_pending_rule_bundle_view(item));
    }
    views
}

pub fn sum_pending_rule_bundle_view(view: &PendingRuleBundleView) -> Result<f64, String> {
    let parsed_quota_units = view
        .quota_units_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    Ok(parsed_quota_units as f64 + parsed_ratio)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_pending_rule_bundle("code-554", "title-383");
        let view = to_pending_rule_bundle_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_pending_rule_bundle_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
