use super::*;

use acme_kit::text;

/// How screens and exports show a credential template.
#[derive(Debug, Clone, PartialEq)]
pub struct CredentialTemplateView {
    pub code: String,
    pub name: String,
    pub maximum: String,
    pub cluster: String,
    pub ratio: String,
    pub currency_text: String,
    pub version_text: String,
    pub audience: String,
    pub locale: String,
    pub updated_at_text: String,
    pub window_days: String,
    pub status_label: String,
}

pub fn to_credential_template_view(item: &CredentialTemplate) -> CredentialTemplateView {
    CredentialTemplateView {
        code: text::truncate(&item.code, 120),
        name: item.name.clone(),
        maximum: format!("{:.2}", item.maximum),
        cluster: item.cluster.clone(),
        ratio: format!("{:.2}", item.ratio),
        currency_text: text::truncate(&item.currency, 40),
        version_text: item.version.to_string(),
        audience: item.audience.clone(),
        locale: item.locale.clone(),
        updated_at_text: item.updated_at.to_string(),
        window_days: item.window_days.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_credential_template_views(items: &[CredentialTemplate]) -> Vec<CredentialTemplateView> {
    let mut views: Vec<CredentialTemplateView> = Vec::new();
    for item in items.iter() {
        views.push(to_credential_template_view(item));
    }
    views
}

pub fn sum_credential_template_view(view: &CredentialTemplateView) -> Result<f64, String> {
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    let parsed_version = view
        .version_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    Ok(parsed_maximum + parsed_ratio + (parsed_version as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_credential_template("code-746", "name-999");
        let view = to_credential_template_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_credential_template_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
