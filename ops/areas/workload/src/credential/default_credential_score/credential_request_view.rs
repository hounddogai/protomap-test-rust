use super::*;

use acme_kit::text;

/// How screens and exports show a credential request.
#[derive(Debug, Clone, PartialEq)]
pub struct CredentialRequestView {
    pub code_text: String,
    pub label_text: String,
    pub rate: String,
    pub phone: String,
    pub batch_size_text: String,
    pub floor_value_text: String,
    pub sequence: String,
    pub cipher: String,
    pub attempts: String,
    pub port: String,
    pub rotation_days_text: String,
    pub status_label: String,
}

pub fn to_credential_request_view(item: &CredentialRequest) -> CredentialRequestView {
    CredentialRequestView {
        code_text: item.code.clone(),
        label_text: item.label.clone(),
        rate: format!("{:.2}", item.rate),
        phone: item.phone.clone(),
        batch_size_text: item.batch_size.to_string(),
        floor_value_text: format!("{:.2}", item.floor_value),
        sequence: item.sequence.to_string(),
        cipher: text::truncate(&item.cipher, 64),
        attempts: item.attempts.to_string(),
        port: item.port.to_string(),
        rotation_days_text: item.rotation_days.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_credential_request_views(items: &[CredentialRequest]) -> Vec<CredentialRequestView> {
    let mut views: Vec<CredentialRequestView> = Vec::new();
    for item in items.iter() {
        views.push(to_credential_request_view(item));
    }
    views
}

pub fn sum_credential_request_view(view: &CredentialRequestView) -> Result<f64, String> {
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    let parsed_batch_size = view
        .batch_size_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    let parsed_floor_value = view
        .floor_value_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    Ok(parsed_rate + (parsed_batch_size as f64) + parsed_floor_value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_credential_request("code-156", "label-900");
        let view = to_credential_request_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_credential_request_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
