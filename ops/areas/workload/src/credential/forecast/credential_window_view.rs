use super::*;

use acme_kit::text;

/// How screens and exports show a credential window.
#[derive(Debug, Clone, PartialEq)]
pub struct CredentialWindowView {
    pub code: String,
    pub name_text: String,
    pub phone: String,
    pub ratio: String,
    pub duration_minutes_text: String,
    pub rate_text: String,
    pub region: String,
    pub retries: String,
    pub scope_text: String,
    pub status_label: String,
}

pub fn to_credential_window_view(item: &CredentialWindow) -> CredentialWindowView {
    CredentialWindowView {
        code: item.code.clone(),
        name_text: text::truncate(&item.name, 120),
        phone: item.phone.clone(),
        ratio: format!("{:.2}", item.ratio),
        duration_minutes_text: item.duration_minutes.to_string(),
        rate_text: format!("{:.2}", item.rate),
        region: text::truncate(&item.region, 40),
        retries: item.retries.to_string(),
        scope_text: item.scope.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_credential_window_views(items: &[CredentialWindow]) -> Vec<CredentialWindowView> {
    let mut views: Vec<CredentialWindowView> = Vec::new();
    for item in items.iter() {
        views.push(to_credential_window_view(item));
    }
    views
}

pub fn sum_credential_window_view(view: &CredentialWindowView) -> Result<f64, String> {
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    let parsed_duration_minutes = view
        .duration_minutes_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    let parsed_rate = view
        .rate_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    Ok(parsed_ratio + (parsed_duration_minutes as f64) + parsed_rate)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_credential_window("code-197", "name-884");
        let view = to_credential_window_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_credential_window_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
