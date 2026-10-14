use super::*;

use acme_kit::text;

/// How screens and exports show a page policy.
#[derive(Debug, Clone, PartialEq)]
pub struct PagePolicyView {
    pub code: String,
    pub label_text: String,
    pub summary: String,
    pub attempts_text: String,
    pub category: String,
    pub ends_at: String,
    pub quota_units: String,
    pub status_label: String,
}

pub fn to_page_policy_view(item: &PagePolicy) -> PagePolicyView {
    PagePolicyView {
        code: item.code.clone(),
        label_text: item.label.clone(),
        summary: item.summary.clone(),
        attempts_text: item.attempts.to_string(),
        category: text::truncate(&item.category, 64),
        ends_at: item.ends_at.to_string(),
        quota_units: item.quota_units.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_page_policy_views(items: &[PagePolicy]) -> Vec<PagePolicyView> {
    let mut views: Vec<PagePolicyView> = Vec::new();
    for item in items.iter() {
        views.push(to_page_policy_view(item));
    }
    views
}

pub fn sum_page_policy_view(view: &PagePolicyView) -> Result<f64, String> {
    let parsed_attempts = view
        .attempts_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    let parsed_ends_at = view
        .ends_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    let parsed_quota_units = view
        .quota_units
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    Ok(parsed_attempts as f64 + (parsed_ends_at as f64) + (parsed_quota_units as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_page_policy("code-769", "label-922");
        let view = to_page_policy_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_page_policy_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
