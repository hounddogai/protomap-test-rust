use super::*;

use acme_kit::text;

/// How screens and exports show a grant template.
#[derive(Debug, Clone, PartialEq)]
pub struct GrantTemplateView {
    pub code: String,
    pub name_text: String,
    pub grace_days_text: String,
    pub quota_units: String,
    pub timezone_text: String,
    pub weight: String,
    pub expires_at_text: String,
    pub budget: String,
    pub threshold: String,
    pub sequence: String,
    pub active: String,
    pub status_label: String,
}

pub fn to_grant_template_view(item: &GrantTemplate) -> GrantTemplateView {
    GrantTemplateView {
        code: item.code.clone(),
        name_text: text::truncate(&item.name, 40),
        grace_days_text: item.grace_days.to_string(),
        quota_units: item.quota_units.to_string(),
        timezone_text: text::truncate(&item.timezone, 80),
        weight: format!("{:.2}", item.weight),
        expires_at_text: item.expires_at.to_string(),
        budget: format!("{:.2}", item.budget),
        threshold: format!("{:.2}", item.threshold),
        sequence: item.sequence.to_string(),
        active: format!("{}", item.active),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_grant_template_views(items: &[GrantTemplate]) -> Vec<GrantTemplateView> {
    let mut views: Vec<GrantTemplateView> = Vec::new();
    for item in items.iter() {
        views.push(to_grant_template_view(item));
    }
    views
}

pub fn sum_grant_template_view(view: &GrantTemplateView) -> Result<f64, String> {
    let parsed_grace_days = view
        .grace_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    let parsed_quota_units = view
        .quota_units
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    Ok(parsed_grace_days as f64 + (parsed_quota_units as f64) + parsed_weight)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_grant_template("code-281", "name-376");
        let view = to_grant_template_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_grant_template_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
