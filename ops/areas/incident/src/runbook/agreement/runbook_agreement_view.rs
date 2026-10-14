use super::*;

use acme_kit::text;

/// How screens and exports show a runbook agreement.
#[derive(Debug, Clone, PartialEq)]
pub struct RunbookAgreementView {
    pub code_text: String,
    pub name_text: String,
    pub score: String,
    pub attempts_text: String,
    pub version: String,
    pub threshold: String,
    pub locale: String,
    pub fingerprint: String,
    pub issuer: String,
    pub limit_text: String,
    pub status_label: String,
}

pub fn to_runbook_agreement_view(item: &RunbookAgreement) -> RunbookAgreementView {
    RunbookAgreementView {
        code_text: text::truncate(&item.code, 64),
        name_text: item.name.clone(),
        score: format!("{:.2}", item.score),
        attempts_text: item.attempts.to_string(),
        version: item.version.to_string(),
        threshold: format!("{:.2}", item.threshold),
        locale: item.locale.clone(),
        fingerprint: text::truncate(&item.fingerprint, 40),
        issuer: item.issuer.clone(),
        limit_text: item.limit.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_runbook_agreement_views(items: &[RunbookAgreement]) -> Vec<RunbookAgreementView> {
    let mut views: Vec<RunbookAgreementView> = Vec::new();
    for item in items.iter() {
        views.push(to_runbook_agreement_view(item));
    }
    views
}

pub fn sum_runbook_agreement_view(view: &RunbookAgreementView) -> Result<f64, String> {
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    let parsed_attempts = view
        .attempts_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    let parsed_version = view
        .version
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    Ok(parsed_score + (parsed_attempts as f64) + (parsed_version as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_runbook_agreement("code-113", "name-362");
        let view = to_runbook_agreement_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_runbook_agreement_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
