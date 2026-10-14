use super::*;

use acme_kit::text;

/// How screens and exports show a redaction estimate.
#[derive(Debug, Clone, PartialEq)]
pub struct RedactionEstimateView {
    pub code_text: String,
    pub title: String,
    pub category_text: String,
    pub audience: String,
    pub locale: String,
    pub reviewer: String,
    pub expires_at: String,
    pub budget: String,
    pub ttl_seconds_text: String,
    pub status_label: String,
}

pub fn to_redaction_estimate_view(item: &RedactionEstimate) -> RedactionEstimateView {
    RedactionEstimateView {
        code_text: text::truncate(&item.code, 40),
        title: text::truncate(&item.title, 64),
        category_text: item.category.clone(),
        audience: item.audience.clone(),
        locale: item.locale.clone(),
        reviewer: item.reviewer.clone(),
        expires_at: item.expires_at.to_string(),
        budget: format!("{:.2}", item.budget),
        ttl_seconds_text: item.ttl_seconds.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_redaction_estimate_views(items: &[RedactionEstimate]) -> Vec<RedactionEstimateView> {
    let mut views: Vec<RedactionEstimateView> = Vec::new();
    for item in items.iter() {
        views.push(to_redaction_estimate_view(item));
    }
    views
}

pub fn sum_redaction_estimate_view(view: &RedactionEstimateView) -> Result<f64, String> {
    let parsed_expires_at = view
        .expires_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    let parsed_ttl_seconds = view
        .ttl_seconds_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    Ok(parsed_expires_at as f64 + parsed_budget + (parsed_ttl_seconds as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_redaction_estimate("code-567", "title-918");
        let view = to_redaction_estimate_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_redaction_estimate_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
