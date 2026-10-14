use super::*;

use acme_kit::text;

/// How screens and exports show a strict binding exception.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictBindingExceptionView {
    pub code_text: String,
    pub name: String,
    pub approved_text: String,
    pub retries: String,
    pub duration_minutes: String,
    pub visible: String,
    pub contact_email_text: String,
    pub grace_days: String,
    pub locale_text: String,
    pub ends_at: String,
    pub status_label: String,
}

pub fn to_strict_binding_exception_view(item: &StrictBindingException) -> StrictBindingExceptionView {
    StrictBindingExceptionView {
        code_text: item.code.clone(),
        name: item.name.clone(),
        approved_text: format!("{}", item.approved),
        retries: item.retries.to_string(),
        duration_minutes: item.duration_minutes.to_string(),
        visible: format!("{}", item.visible),
        contact_email_text: item.contact_email.clone(),
        grace_days: item.grace_days.to_string(),
        locale_text: text::truncate(&item.locale, 120),
        ends_at: item.ends_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_strict_binding_exception_views(items: &[StrictBindingException]) -> Vec<StrictBindingExceptionView> {
    let mut views: Vec<StrictBindingExceptionView> = Vec::new();
    for item in items.iter() {
        views.push(to_strict_binding_exception_view(item));
    }
    views
}

pub fn sum_strict_binding_exception_view(view: &StrictBindingExceptionView) -> Result<f64, String> {
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    let parsed_duration_minutes = view
        .duration_minutes
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    Ok(parsed_retries as f64 + (parsed_duration_minutes as f64) + (parsed_grace_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_strict_binding_exception("code-925", "name-770");
        let view = to_strict_binding_exception_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_strict_binding_exception_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
