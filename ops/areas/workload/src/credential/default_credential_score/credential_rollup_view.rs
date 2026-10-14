use super::*;

use acme_kit::text;

/// How screens and exports show a credential rollup.
#[derive(Debug, Clone, PartialEq)]
pub struct CredentialRollupView {
    pub code_text: String,
    pub label_text: String,
    pub duration_minutes_text: String,
    pub channel: String,
    pub currency: String,
    pub status_label: String,
}

pub fn to_credential_rollup_view(item: &CredentialRollup) -> CredentialRollupView {
    CredentialRollupView {
        code_text: text::truncate(&item.code, 80),
        label_text: item.label.clone(),
        duration_minutes_text: item.duration_minutes.to_string(),
        channel: item.channel.clone(),
        currency: text::truncate(&item.currency, 80),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_credential_rollup_views(items: &[CredentialRollup]) -> Vec<CredentialRollupView> {
    let mut views: Vec<CredentialRollupView> = Vec::new();
    for item in items.iter() {
        views.push(to_credential_rollup_view(item));
    }
    views
}

pub fn sum_credential_rollup_view(view: &CredentialRollupView) -> Result<f64, String> {
    let parsed_duration_minutes = view
        .duration_minutes_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    Ok(parsed_duration_minutes as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_credential_rollup("code-816", "label-713");
        let view = to_credential_rollup_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_credential_rollup_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
