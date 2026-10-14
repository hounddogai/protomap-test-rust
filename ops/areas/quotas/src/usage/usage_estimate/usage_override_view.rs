use super::*;

use acme_kit::text;

/// How screens and exports show a usage override.
#[derive(Debug, Clone, PartialEq)]
pub struct UsageOverrideView {
    pub code: String,
    pub label: String,
    pub audience_text: String,
    pub duration_minutes: String,
    pub ends_at_text: String,
    pub category: String,
    pub locale: String,
    pub starts_at: String,
    pub approved_text: String,
    pub namespace: String,
    pub status_label: String,
}

pub fn to_usage_override_view(item: &UsageOverride) -> UsageOverrideView {
    UsageOverrideView {
        code: item.code.clone(),
        label: item.label.clone(),
        audience_text: text::truncate(&item.audience, 64),
        duration_minutes: item.duration_minutes.to_string(),
        ends_at_text: item.ends_at.to_string(),
        category: item.category.clone(),
        locale: item.locale.clone(),
        starts_at: item.starts_at.to_string(),
        approved_text: format!("{}", item.approved),
        namespace: item.namespace.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_usage_override_views(items: &[UsageOverride]) -> Vec<UsageOverrideView> {
    let mut views: Vec<UsageOverrideView> = Vec::new();
    for item in items.iter() {
        views.push(to_usage_override_view(item));
    }
    views
}

pub fn sum_usage_override_view(view: &UsageOverrideView) -> Result<f64, String> {
    let parsed_duration_minutes = view
        .duration_minutes
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    let parsed_ends_at = view
        .ends_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    Ok(parsed_duration_minutes as f64 + (parsed_ends_at as f64) + (parsed_starts_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_usage_override("code-358", "label-727");
        let view = to_usage_override_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_usage_override_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
