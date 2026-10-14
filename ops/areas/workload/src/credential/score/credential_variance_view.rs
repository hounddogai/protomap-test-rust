use super::*;

use acme_kit::text;

/// How screens and exports show a credential variance.
#[derive(Debug, Clone, PartialEq)]
pub struct CredentialVarianceView {
    pub code: String,
    pub title: String,
    pub country: String,
    pub rate_text: String,
    pub locale_text: String,
    pub name: String,
    pub phone_text: String,
    pub rotation_days: String,
    pub limit_text: String,
    pub segment: String,
    pub status_label: String,
}

pub fn to_credential_variance_view(item: &CredentialVariance) -> CredentialVarianceView {
    CredentialVarianceView {
        code: text::truncate(&item.code, 40),
        title: item.title.clone(),
        country: item.country.clone(),
        rate_text: format!("{:.2}", item.rate),
        locale_text: text::truncate(&item.locale, 64),
        name: item.name.clone(),
        phone_text: item.phone.clone(),
        rotation_days: item.rotation_days.to_string(),
        limit_text: item.limit.to_string(),
        segment: item.segment.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_credential_variance_views(items: &[CredentialVariance]) -> Vec<CredentialVarianceView> {
    let mut views: Vec<CredentialVarianceView> = Vec::new();
    for item in items.iter() {
        views.push(to_credential_variance_view(item));
    }
    views
}

pub fn sum_credential_variance_view(view: &CredentialVarianceView) -> Result<f64, String> {
    let parsed_rate = view
        .rate_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    let parsed_rotation_days = view
        .rotation_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    let parsed_limit = view
        .limit_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    Ok(parsed_rate + (parsed_rotation_days as f64) + (parsed_limit as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_credential_variance("code-276", "title-112");
        let view = to_credential_variance_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_credential_variance_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
