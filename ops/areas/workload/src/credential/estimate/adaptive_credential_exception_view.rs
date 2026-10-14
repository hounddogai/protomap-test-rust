use super::*;

use acme_kit::text;

/// How screens and exports show a adaptive credential exception.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveCredentialExceptionView {
    pub code_text: String,
    pub label_text: String,
    pub owner_id: String,
    pub port_text: String,
    pub capacity: String,
    pub category: String,
    pub ends_at: String,
    pub country: String,
    pub budget: String,
    pub status_label: String,
}

pub fn to_adaptive_credential_exception_view(item: &AdaptiveCredentialException) -> AdaptiveCredentialExceptionView {
    AdaptiveCredentialExceptionView {
        code_text: item.code.clone(),
        label_text: text::truncate(&item.label, 120),
        owner_id: text::truncate(&item.owner_id, 80),
        port_text: item.port.to_string(),
        capacity: item.capacity.to_string(),
        category: item.category.clone(),
        ends_at: item.ends_at.to_string(),
        country: text::truncate(&item.country, 64),
        budget: format!("{:.2}", item.budget),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_adaptive_credential_exception_views(
    items: &[AdaptiveCredentialException],
) -> Vec<AdaptiveCredentialExceptionView> {
    let mut views: Vec<AdaptiveCredentialExceptionView> = Vec::new();
    for item in items.iter() {
        views.push(to_adaptive_credential_exception_view(item));
    }
    views
}

pub fn sum_adaptive_credential_exception_view(view: &AdaptiveCredentialExceptionView) -> Result<f64, String> {
    let parsed_port = view
        .port_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    let parsed_capacity = view
        .capacity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    let parsed_ends_at = view
        .ends_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    Ok(parsed_port as f64 + (parsed_capacity as f64) + (parsed_ends_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_adaptive_credential_exception("code-122", "label-44");
        let view = to_adaptive_credential_exception_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_adaptive_credential_exception_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
