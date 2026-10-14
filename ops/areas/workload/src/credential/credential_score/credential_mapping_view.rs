use super::*;

use acme_kit::text;

/// How screens and exports show a credential mapping.
#[derive(Debug, Clone, PartialEq)]
pub struct CredentialMappingView {
    pub code_text: String,
    pub name: String,
    pub burst_text: String,
    pub margin_text: String,
    pub issuer: String,
    pub weight: String,
    pub label: String,
    pub approved_text: String,
    pub status_label: String,
}

pub fn to_credential_mapping_view(item: &CredentialMapping) -> CredentialMappingView {
    CredentialMappingView {
        code_text: item.code.clone(),
        name: item.name.clone(),
        burst_text: item.burst.to_string(),
        margin_text: format!("{:.2}", item.margin),
        issuer: text::truncate(&item.issuer, 80),
        weight: format!("{:.2}", item.weight),
        label: text::truncate(&item.label, 80),
        approved_text: format!("{}", item.approved),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_credential_mapping_views(items: &[CredentialMapping]) -> Vec<CredentialMappingView> {
    let mut views: Vec<CredentialMappingView> = Vec::new();
    for item in items.iter() {
        views.push(to_credential_mapping_view(item));
    }
    views
}

pub fn sum_credential_mapping_view(view: &CredentialMappingView) -> Result<f64, String> {
    let parsed_burst = view
        .burst_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    let parsed_margin = view
        .margin_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    Ok(parsed_burst as f64 + parsed_margin + parsed_weight)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_credential_mapping("code-291", "name-125");
        let view = to_credential_mapping_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_credential_mapping_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
