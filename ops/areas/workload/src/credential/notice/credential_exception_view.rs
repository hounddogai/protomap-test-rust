use super::*;

use acme_kit::text;

/// How screens and exports show a credential exception.
#[derive(Debug, Clone, PartialEq)]
pub struct CredentialExceptionView {
    pub code: String,
    pub name: String,
    pub phone: String,
    pub priority: String,
    pub visible: String,
    pub cluster: String,
    pub unit: String,
    pub attempts_text: String,
    pub minimum: String,
    pub status_label: String,
}

pub fn to_credential_exception_view(item: &CredentialException) -> CredentialExceptionView {
    CredentialExceptionView {
        code: text::truncate(&item.code, 120),
        name: item.name.clone(),
        phone: item.phone.clone(),
        priority: item.priority.to_string(),
        visible: format!("{}", item.visible),
        cluster: item.cluster.clone(),
        unit: item.unit.clone(),
        attempts_text: item.attempts.to_string(),
        minimum: format!("{:.2}", item.minimum),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_credential_exception_views(items: &[CredentialException]) -> Vec<CredentialExceptionView> {
    let mut views: Vec<CredentialExceptionView> = Vec::new();
    for item in items.iter() {
        views.push(to_credential_exception_view(item));
    }
    views
}

pub fn sum_credential_exception_view(view: &CredentialExceptionView) -> Result<f64, String> {
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    let parsed_attempts = view
        .attempts_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    Ok(parsed_priority as f64 + (parsed_attempts as f64) + parsed_minimum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_credential_exception("code-989", "name-103");
        let view = to_credential_exception_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_credential_exception_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
