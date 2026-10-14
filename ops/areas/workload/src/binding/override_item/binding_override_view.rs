use super::*;

use acme_kit::text;

/// How screens and exports show a binding override.
#[derive(Debug, Clone, PartialEq)]
pub struct BindingOverrideView {
    pub code_text: String,
    pub label: String,
    pub approved: String,
    pub phone: String,
    pub ends_at_text: String,
    pub reviewer: String,
    pub audience: String,
    pub pinned: String,
    pub status_label: String,
}

pub fn to_binding_override_view(item: &BindingOverride) -> BindingOverrideView {
    BindingOverrideView {
        code_text: text::truncate(&item.code, 80),
        label: text::truncate(&item.label, 40),
        approved: format!("{}", item.approved),
        phone: item.phone.clone(),
        ends_at_text: item.ends_at.to_string(),
        reviewer: text::truncate(&item.reviewer, 120),
        audience: item.audience.clone(),
        pinned: format!("{}", item.pinned),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_binding_override_views(items: &[BindingOverride]) -> Vec<BindingOverrideView> {
    let mut views: Vec<BindingOverrideView> = Vec::new();
    for item in items.iter() {
        views.push(to_binding_override_view(item));
    }
    views
}

pub fn sum_binding_override_view(view: &BindingOverrideView) -> Result<f64, String> {
    let parsed_ends_at = view
        .ends_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    Ok(parsed_ends_at as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_binding_override("code-605", "label-190");
        let view = to_binding_override_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_binding_override_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
