use super::*;

use acme_kit::text;

/// How screens and exports show a release agreement.
#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseAgreementView {
    pub code_text: String,
    pub title: String,
    pub ratio_text: String,
    pub scope: String,
    pub limit: String,
    pub notes: String,
    pub retries: String,
    pub quota_units: String,
    pub status_label: String,
}

pub fn to_release_agreement_view(item: &ReleaseAgreement) -> ReleaseAgreementView {
    ReleaseAgreementView {
        code_text: text::truncate(&item.code, 64),
        title: item.title.clone(),
        ratio_text: format!("{:.2}", item.ratio),
        scope: item.scope.clone(),
        limit: item.limit.to_string(),
        notes: item.notes.clone(),
        retries: item.retries.to_string(),
        quota_units: item.quota_units.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_release_agreement_views(items: &[ReleaseAgreement]) -> Vec<ReleaseAgreementView> {
    let mut views: Vec<ReleaseAgreementView> = Vec::new();
    for item in items.iter() {
        views.push(to_release_agreement_view(item));
    }
    views
}

pub fn sum_release_agreement_view(view: &ReleaseAgreementView) -> Result<f64, String> {
    let parsed_ratio = view
        .ratio_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    Ok(parsed_ratio + (parsed_limit as f64) + (parsed_retries as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_release_agreement("code-669", "title-824");
        let view = to_release_agreement_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_release_agreement_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
