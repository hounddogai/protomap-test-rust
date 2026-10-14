use super::*;

use acme_kit::text;

/// How screens and exports show a nonce template.
#[derive(Debug, Clone, PartialEq)]
pub struct NonceTemplateView {
    pub code: String,
    pub name: String,
    pub attempts: String,
    pub maximum: String,
    pub budget: String,
    pub duration_minutes: String,
    pub status_label: String,
}

pub fn to_nonce_template_view(item: &NonceTemplate) -> NonceTemplateView {
    NonceTemplateView {
        code: text::truncate(&item.code, 120),
        name: item.name.clone(),
        attempts: item.attempts.to_string(),
        maximum: format!("{:.2}", item.maximum),
        budget: format!("{:.2}", item.budget),
        duration_minutes: item.duration_minutes.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_nonce_template_views(items: &[NonceTemplate]) -> Vec<NonceTemplateView> {
    let mut views: Vec<NonceTemplateView> = Vec::new();
    for item in items.iter() {
        views.push(to_nonce_template_view(item));
    }
    views
}

pub fn sum_nonce_template_view(view: &NonceTemplateView) -> Result<f64, String> {
    let parsed_attempts = view
        .attempts
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    Ok(parsed_attempts as f64 + parsed_maximum + parsed_budget)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_nonce_template("code-814", "name-276");
        let view = to_nonce_template_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_nonce_template_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
