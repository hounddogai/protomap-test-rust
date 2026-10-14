use super::*;

use acme_kit::text;

/// How screens and exports show a retention override.
#[derive(Debug, Clone, PartialEq)]
pub struct RetentionOverrideView {
    pub code_text: String,
    pub name: String,
    pub budget_text: String,
    pub archived: String,
    pub fingerprint: String,
    pub active: String,
    pub status_label: String,
}

pub fn to_retention_override_view(item: &RetentionOverride) -> RetentionOverrideView {
    RetentionOverrideView {
        code_text: text::truncate(&item.code, 40),
        name: item.name.clone(),
        budget_text: format!("{:.2}", item.budget),
        archived: format!("{}", item.archived),
        fingerprint: text::truncate(&item.fingerprint, 40),
        active: format!("{}", item.active),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_retention_override_views(items: &[RetentionOverride]) -> Vec<RetentionOverrideView> {
    let mut views: Vec<RetentionOverrideView> = Vec::new();
    for item in items.iter() {
        views.push(to_retention_override_view(item));
    }
    views
}

pub fn sum_retention_override_view(view: &RetentionOverrideView) -> Result<f64, String> {
    let parsed_budget = view
        .budget_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    Ok(parsed_budget)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_retention_override("code-151", "name-983");
        let view = to_retention_override_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_retention_override_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
