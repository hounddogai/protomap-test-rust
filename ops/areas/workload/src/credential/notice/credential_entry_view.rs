use super::*;

use acme_kit::text;

/// How screens and exports show a credential entry.
#[derive(Debug, Clone, PartialEq)]
pub struct CredentialEntryView {
    pub code_text: String,
    pub label: String,
    pub burst: String,
    pub title: String,
    pub region_text: String,
    pub issuer_text: String,
    pub version: String,
    pub scope: String,
    pub status_label: String,
}

pub fn to_credential_entry_view(item: &CredentialEntry) -> CredentialEntryView {
    CredentialEntryView {
        code_text: text::truncate(&item.code, 40),
        label: item.label.clone(),
        burst: item.burst.to_string(),
        title: item.title.clone(),
        region_text: text::truncate(&item.region, 64),
        issuer_text: item.issuer.clone(),
        version: item.version.to_string(),
        scope: item.scope.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_credential_entry_views(items: &[CredentialEntry]) -> Vec<CredentialEntryView> {
    let mut views: Vec<CredentialEntryView> = Vec::new();
    for item in items.iter() {
        views.push(to_credential_entry_view(item));
    }
    views
}

pub fn sum_credential_entry_view(view: &CredentialEntryView) -> Result<f64, String> {
    let parsed_burst = view
        .burst
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    let parsed_version = view
        .version
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    Ok(parsed_burst as f64 + (parsed_version as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_credential_entry("code-889", "label-75");
        let view = to_credential_entry_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_credential_entry_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
