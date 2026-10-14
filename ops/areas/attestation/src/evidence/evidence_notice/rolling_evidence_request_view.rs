use super::*;

use acme_kit::text;

/// How screens and exports show a rolling evidence request.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingEvidenceRequestView {
    pub code_text: String,
    pub title_text: String,
    pub notes: String,
    pub attempts: String,
    pub audience: String,
    pub locale: String,
    pub status_label: String,
}

pub fn to_rolling_evidence_request_view(item: &RollingEvidenceRequest) -> RollingEvidenceRequestView {
    RollingEvidenceRequestView {
        code_text: text::truncate(&item.code, 40),
        title_text: item.title.clone(),
        notes: item.notes.clone(),
        attempts: item.attempts.to_string(),
        audience: item.audience.clone(),
        locale: item.locale.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rolling_evidence_request_views(items: &[RollingEvidenceRequest]) -> Vec<RollingEvidenceRequestView> {
    let mut views: Vec<RollingEvidenceRequestView> = Vec::new();
    for item in items.iter() {
        views.push(to_rolling_evidence_request_view(item));
    }
    views
}

pub fn sum_rolling_evidence_request_view(view: &RollingEvidenceRequestView) -> Result<f64, String> {
    let parsed_attempts = view
        .attempts
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    Ok(parsed_attempts as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_rolling_evidence_request("code-422", "title-79");
        let view = to_rolling_evidence_request_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_rolling_evidence_request_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
