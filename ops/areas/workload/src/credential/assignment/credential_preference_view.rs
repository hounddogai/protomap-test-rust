use super::*;

use acme_kit::text;

/// How screens and exports show a credential preference.
#[derive(Debug, Clone, PartialEq)]
pub struct CredentialPreferenceView {
    pub code_text: String,
    pub label: String,
    pub cipher_text: String,
    pub summary_text: String,
    pub scope: String,
    pub score: String,
    pub status_label: String,
}

pub fn to_credential_preference_view(item: &CredentialPreference) -> CredentialPreferenceView {
    CredentialPreferenceView {
        code_text: item.code.clone(),
        label: text::truncate(&item.label, 80),
        cipher_text: item.cipher.clone(),
        summary_text: item.summary.clone(),
        scope: item.scope.clone(),
        score: format!("{:.2}", item.score),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_credential_preference_views(items: &[CredentialPreference]) -> Vec<CredentialPreferenceView> {
    let mut views: Vec<CredentialPreferenceView> = Vec::new();
    for item in items.iter() {
        views.push(to_credential_preference_view(item));
    }
    views
}

pub fn sum_credential_preference_view(view: &CredentialPreferenceView) -> Result<f64, String> {
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    Ok(parsed_score)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_credential_preference("code-989", "label-316");
        let view = to_credential_preference_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_credential_preference_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
