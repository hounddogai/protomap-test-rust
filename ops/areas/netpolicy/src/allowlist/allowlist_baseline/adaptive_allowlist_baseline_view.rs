use super::*;

use acme_kit::text;

/// How screens and exports show a adaptive allowlist baseline.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveAllowlistBaselineView {
    pub code_text: String,
    pub title: String,
    pub priority: String,
    pub retries: String,
    pub discount: String,
    pub country: String,
    pub version: String,
    pub description_text: String,
    pub status_label: String,
}

pub fn to_adaptive_allowlist_baseline_view(item: &AdaptiveAllowlistBaseline) -> AdaptiveAllowlistBaselineView {
    AdaptiveAllowlistBaselineView {
        code_text: item.code.clone(),
        title: text::truncate(&item.title, 40),
        priority: item.priority.to_string(),
        retries: item.retries.to_string(),
        discount: format!("{:.2}", item.discount),
        country: text::truncate(&item.country, 120),
        version: item.version.to_string(),
        description_text: item.description.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_adaptive_allowlist_baseline_views(items: &[AdaptiveAllowlistBaseline]) -> Vec<AdaptiveAllowlistBaselineView> {
    let mut views: Vec<AdaptiveAllowlistBaselineView> = Vec::new();
    for item in items.iter() {
        views.push(to_adaptive_allowlist_baseline_view(item));
    }
    views
}

pub fn sum_adaptive_allowlist_baseline_view(view: &AdaptiveAllowlistBaselineView) -> Result<f64, String> {
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    Ok(parsed_priority as f64 + (parsed_retries as f64) + parsed_discount)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_adaptive_allowlist_baseline("code-169", "title-849");
        let view = to_adaptive_allowlist_baseline_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_adaptive_allowlist_baseline_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
