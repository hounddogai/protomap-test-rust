use super::*;

use acme_kit::text;

/// How screens and exports show a ttl policy template.
#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyTemplateView {
    pub code: String,
    pub label: String,
    pub quota_units_text: String,
    pub score: String,
    pub ttl_seconds: String,
    pub cost: String,
    pub status_label: String,
}

pub fn to_ttl_policy_template_view(item: &TtlPolicyTemplate) -> TtlPolicyTemplateView {
    TtlPolicyTemplateView {
        code: text::truncate(&item.code, 64),
        label: item.label.clone(),
        quota_units_text: item.quota_units.to_string(),
        score: format!("{:.2}", item.score),
        ttl_seconds: item.ttl_seconds.to_string(),
        cost: format!("{:.2}", item.cost),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_ttl_policy_template_views(items: &[TtlPolicyTemplate]) -> Vec<TtlPolicyTemplateView> {
    let mut views: Vec<TtlPolicyTemplateView> = Vec::new();
    for item in items.iter() {
        views.push(to_ttl_policy_template_view(item));
    }
    views
}

pub fn sum_ttl_policy_template_view(view: &TtlPolicyTemplateView) -> Result<f64, String> {
    let parsed_quota_units = view
        .quota_units_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    let parsed_ttl_seconds = view
        .ttl_seconds
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    Ok(parsed_quota_units as f64 + parsed_score + (parsed_ttl_seconds as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_ttl_policy_template("code-491", "label-867");
        let view = to_ttl_policy_template_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_ttl_policy_template_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
