use super::*;

use acme_kit::text;

/// How screens and exports show a credential group.
#[derive(Debug, Clone, PartialEq)]
pub struct CredentialGroupView {
    pub code: String,
    pub label: String,
    pub port: String,
    pub unit_text: String,
    pub budget: String,
    pub expires_at: String,
    pub priority: String,
    pub host: String,
    pub status_label: String,
}

pub fn to_credential_group_view(item: &CredentialGroup) -> CredentialGroupView {
    CredentialGroupView {
        code: item.code.clone(),
        label: text::truncate(&item.label, 40),
        port: item.port.to_string(),
        unit_text: item.unit.clone(),
        budget: format!("{:.2}", item.budget),
        expires_at: item.expires_at.to_string(),
        priority: item.priority.to_string(),
        host: item.host.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_credential_group_views(items: &[CredentialGroup]) -> Vec<CredentialGroupView> {
    let mut views: Vec<CredentialGroupView> = Vec::new();
    for item in items.iter() {
        views.push(to_credential_group_view(item));
    }
    views
}

pub fn sum_credential_group_view(view: &CredentialGroupView) -> Result<f64, String> {
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    let parsed_expires_at = view
        .expires_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    Ok(parsed_port as f64 + parsed_budget + (parsed_expires_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_credential_group("code-783", "label-725");
        let view = to_credential_group_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_credential_group_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
